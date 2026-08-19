use std::env;
use std::io::{self, Write};
use std::path::PathBuf;

use tidyup_core::{
    ActionExecutionStatus, ExecutionReport, Plan, RulePackV1, ValidationReport, build_plan,
    execute_plan, render_plan_json, render_scan_json, scan_root, validate_plan,
};
use tidyup_storage::{default_history_db_path, load_operations, persist_execution};

fn main() {
    match run(env::args().skip(1).collect()) {
        Ok(output) => {
            println!("{output}");
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

fn run(args: Vec<String>) -> Result<String, String> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    run_with_prompt(args, &mut stdin.lock(), &mut stdout.lock())
}

fn run_with_prompt(
    args: Vec<String>,
    input: &mut impl io::BufRead,
    output: &mut impl Write,
) -> Result<String, String> {
    let Some(command) = args.first().map(String::as_str) else {
        return Ok(usage());
    };

    match command {
        "scan" => {
            let options = ReadOnlyCommandOptions::parse(&args[1..])?;
            run_scan(options)
        }
        "plan" => {
            let options = ReadOnlyCommandOptions::parse(&args[1..])?;
            run_plan(options)
        }
        "apply" => {
            let options = ApplyCommandOptions::parse(&args[1..])?;
            run_apply(options, input, output)
        }
        "history" => {
            let options = HistoryCommandOptions::parse(&args[1..])?;
            run_history(options)
        }
        "--help" | "-h" | "help" => Ok(usage()),
        _ => Err(format!("unknown command: {command}\n\n{}", usage())),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutputFormat {
    Human,
    Json,
}

struct ReadOnlyCommandOptions {
    root: PathBuf,
    format: OutputFormat,
}

impl ReadOnlyCommandOptions {
    fn parse(args: &[String]) -> Result<Self, String> {
        let parsed = ParsedCommonArgs::parse(args)?;
        Ok(Self {
            root: parsed.root,
            format: parsed.format,
        })
    }
}

struct ApplyCommandOptions {
    root: PathBuf,
    format: OutputFormat,
    approve: bool,
}

impl ApplyCommandOptions {
    fn parse(args: &[String]) -> Result<Self, String> {
        let parsed = ParsedCommonArgs::parse(args)?;
        Ok(Self {
            root: parsed.root,
            format: parsed.format,
            approve: parsed.approve,
        })
    }
}

struct HistoryCommandOptions {
    root: PathBuf,
    format: OutputFormat,
}

impl HistoryCommandOptions {
    fn parse(args: &[String]) -> Result<Self, String> {
        let parsed = ParsedCommonArgs::parse(args)?;
        Ok(Self {
            root: parsed.root,
            format: parsed.format,
        })
    }
}

struct ParsedCommonArgs {
    root: PathBuf,
    format: OutputFormat,
    approve: bool,
}

impl ParsedCommonArgs {
    fn parse(args: &[String]) -> Result<Self, String> {
        let mut root = None;
        let mut format = OutputFormat::Human;
        let mut approve = false;
        let mut index = 0;

        while index < args.len() {
            match args[index].as_str() {
                "--root" => {
                    let value = args
                        .get(index + 1)
                        .ok_or_else(|| "--root requires a path value".to_owned())?;
                    root = Some(PathBuf::from(value));
                    index += 2;
                }
                "--format" => {
                    let value = args
                        .get(index + 1)
                        .ok_or_else(|| "--format requires `human` or `json`".to_owned())?;
                    format = match value.as_str() {
                        "human" => OutputFormat::Human,
                        "json" => OutputFormat::Json,
                        _ => return Err(format!("unsupported output format: {value}")),
                    };
                    index += 2;
                }
                "--yes" => {
                    approve = true;
                    index += 1;
                }
                "--help" | "-h" => return Err(usage()),
                other => return Err(format!("unexpected argument: {other}\n\n{}", usage())),
            }
        }

        let root = root.unwrap_or(env::current_dir().map_err(|error| error.to_string())?);
        Ok(Self {
            root,
            format,
            approve,
        })
    }
}

fn run_scan(options: ReadOnlyCommandOptions) -> Result<String, String> {
    let scan = scan_root(&options.root).map_err(|error| error.to_string())?;

    Ok(match options.format {
        OutputFormat::Human => render_scan_human(&scan),
        OutputFormat::Json => render_scan_json(&scan),
    })
}

fn run_plan(options: ReadOnlyCommandOptions) -> Result<String, String> {
    let scan = scan_root(&options.root).map_err(|error| error.to_string())?;
    let plan = build_plan(&scan, &RulePackV1::built_in()).map_err(|error| error.to_string())?;
    let validation = validate_plan(&options.root, &plan);

    Ok(match options.format {
        OutputFormat::Human => render_plan_human(&scan.root, &plan, &validation),
        OutputFormat::Json => render_plan_json(&plan, &validation),
    })
}

fn run_apply(
    options: ApplyCommandOptions,
    input: &mut impl io::BufRead,
    output: &mut impl Write,
) -> Result<String, String> {
    let scan = scan_root(&options.root).map_err(|error| error.to_string())?;
    let plan = build_plan(&scan, &RulePackV1::built_in()).map_err(|error| error.to_string())?;
    let validation = validate_plan(&options.root, &plan);

    if !options.approve {
        if options.format == OutputFormat::Json {
            return Ok(render_apply_preview_json(&plan, &validation));
        }

        let preview = render_apply_preview_human(&options.root, &plan, &validation);
        writeln!(output, "{preview}").map_err(|error| error.to_string())?;
        write!(output, "Apply these moves? [y/N]: ").map_err(|error| error.to_string())?;
        output.flush().map_err(|error| error.to_string())?;

        let mut response = String::new();
        input
            .read_line(&mut response)
            .map_err(|error| error.to_string())?;
        if !matches!(response.trim(), "y" | "Y" | "yes" | "YES" | "Yes") {
            return Ok("Apply cancelled. No files were changed.".to_owned());
        }
    }

    let execution = execute_plan(&options.root, &plan);
    let history_db_path = default_history_db_path(&options.root);
    persist_execution(&history_db_path, &plan, &execution).map_err(|error| error.to_string())?;

    Ok(match options.format {
        OutputFormat::Human => {
            render_apply_result_human(&history_db_path, &plan, &validation, &execution)
        }
        OutputFormat::Json => {
            render_apply_result_json(&history_db_path, &plan, &validation, &execution)
        }
    })
}

fn run_history(options: HistoryCommandOptions) -> Result<String, String> {
    let history_db_path = default_history_db_path(&options.root);
    let operations = load_operations(&history_db_path).map_err(|error| error.to_string())?;

    Ok(match options.format {
        OutputFormat::Human => render_history_human(&history_db_path, &operations),
        OutputFormat::Json => render_history_json(&history_db_path, &operations),
    })
}

fn render_scan_human(scan: &tidyup_core::ScanReport) -> String {
    let mut lines = vec![
        format!("Scanned root: {}", scan.root.display()),
        "Read-only scan complete. No files were changed.".to_owned(),
        format!("Supported direct-child files: {}", scan.scanned_files.len()),
        format!("Skipped entries: {}", scan.skipped_entries.len()),
    ];

    if !scan.scanned_files.is_empty() {
        lines.push("Files:".to_owned());
        for file in &scan.scanned_files {
            lines.push(format!(
                "- {} ({} bytes, extension: {})",
                file.relative_path.display(),
                file.size_bytes,
                file.extension.as_deref().unwrap_or("none")
            ));
        }
    }

    if !scan.skipped_entries.is_empty() {
        lines.push("Skipped:".to_owned());
        for entry in &scan.skipped_entries {
            lines.push(format!(
                "- {} [{}]",
                entry.relative_path.display(),
                entry.reason.code()
            ));
        }
    }

    lines.join("\n")
}

fn render_plan_human(root: &std::path::Path, plan: &Plan, validation: &ValidationReport) -> String {
    let mut lines = vec![
        format!("Planned root: {}", root.display()),
        "Read-only plan complete. No files were changed.".to_owned(),
        format!("Plan id: {}", plan.plan_id.as_str()),
        format!("Operation id: {}", plan.operation_id.as_str()),
        format!("Proposed moves: {}", plan.moves.len()),
        format!("Plan skips: {}", plan.skipped_files.len()),
        format!(
            "Validation-ready actions: {}",
            validation.valid_actions.len()
        ),
        format!(
            "Validation rejections: {}",
            validation.invalid_actions.len()
        ),
    ];

    if !plan.moves.is_empty() {
        lines.push("Moves:".to_owned());
        for planned_move in &plan.moves {
            lines.push(format!(
                "- {} -> {} [{}]",
                planned_move.source.relative_path.display(),
                planned_move.destination_relative_path.display(),
                planned_move.rule_id
            ));
        }
    }

    if !plan.skipped_files.is_empty() {
        lines.push("Skipped during planning:".to_owned());
        for skip in &plan.skipped_files {
            lines.push(format!(
                "- {} [{}] {}",
                skip.source_relative_path.display(),
                skip.reason_code.code(),
                skip.detail
            ));
        }
    }

    if !validation.invalid_actions.is_empty() {
        lines.push("Validation issues:".to_owned());
        for action in &validation.invalid_actions {
            lines.push(format!(
                "- {} [{}] {}",
                action.source_relative_path.display(),
                action.reason_code.code(),
                action.detail
            ));
        }
    }

    lines.join("\n")
}

fn render_apply_preview_human(
    root: &std::path::Path,
    plan: &Plan,
    validation: &ValidationReport,
) -> String {
    let mut lines = vec![
        format!("Apply preview for root: {}", root.display()),
        "No files were changed yet.".to_owned(),
        format!("Ready to move: {}", validation.valid_actions.len()),
        format!("Planning skips: {}", plan.skipped_files.len()),
        format!(
            "Validation rejections: {}",
            validation.invalid_actions.len()
        ),
        "Review the proposed moves below, then rerun with --yes to apply them.".to_owned(),
    ];
    if !plan.moves.is_empty() {
        lines.push("Proposed moves:".to_owned());
        for planned_move in &plan.moves {
            lines.push(format!(
                "- {} -> {} [{}]",
                planned_move.source.relative_path.display(),
                planned_move.destination_relative_path.display(),
                planned_move.rule_id
            ));
        }
    }
    if !plan.skipped_files.is_empty() {
        lines.push("Safety skips:".to_owned());
        for skip in &plan.skipped_files {
            lines.push(format!(
                "- {} [{}] {}",
                skip.source_relative_path.display(),
                skip.reason_code.code(),
                skip.detail
            ));
        }
    }
    lines.join("\n")
}

fn render_apply_result_human(
    history_db_path: &std::path::Path,
    plan: &Plan,
    validation: &ValidationReport,
    execution: &ExecutionReport,
) -> String {
    let mut lines = vec![
        format!("Applied root: {}", plan.root.display()),
        format!("Operation id: {}", plan.operation_id.as_str()),
        format!("Plan id: {}", plan.plan_id.as_str()),
        format!("Completed moves: {}", execution.completed_count()),
        format!("Skipped actions: {}", execution.skipped_count()),
        format!("Failed actions: {}", execution.failed_count()),
        format!("Planning skips: {}", plan.skipped_files.len()),
        format!("History database: {}", history_db_path.display()),
    ];
    if !validation.invalid_actions.is_empty() {
        lines.push(format!(
            "Validation rejections before execution: {}",
            validation.invalid_actions.len()
        ));
    }
    if !execution.results.is_empty() {
        lines.push("Apply results:".to_owned());
        for result in &execution.results {
            let status_text = match &result.status {
                ActionExecutionStatus::Completed => "completed".to_owned(),
                ActionExecutionStatus::Skipped {
                    reason_code,
                    detail,
                } => {
                    format!("skipped [{}] {}", reason_code.code(), detail)
                }
                ActionExecutionStatus::Failed { detail } => {
                    format!("failed {}", detail)
                }
            };
            lines.push(format!(
                "- {} -> {} ({})",
                result.source_relative_path.display(),
                result.destination_relative_path.display(),
                status_text
            ));
        }
    }
    if !plan.skipped_files.is_empty() {
        lines.push("Planning skips:".to_owned());
        for skip in &plan.skipped_files {
            lines.push(format!(
                "- {} [{}] {}",
                skip.source_relative_path.display(),
                skip.reason_code.code(),
                skip.detail
            ));
        }
    }
    lines.join("\n")
}

fn render_history_human(
    history_db_path: &std::path::Path,
    operations: &[tidyup_storage::OperationRecord],
) -> String {
    let mut lines = vec![
        format!("History database: {}", history_db_path.display()),
        format!("Recorded operations: {}", operations.len()),
    ];
    for operation in operations {
        lines.push(format!(
            "- {} completed={} skipped={} failed={}",
            operation.operation_id,
            operation.completed_count,
            operation.skipped_count,
            operation.failed_count
        ));
    }
    lines.join("\n")
}

fn render_history_json(
    history_db_path: &std::path::Path,
    operations: &[tidyup_storage::OperationRecord],
) -> String {
    let items = operations
        .iter()
        .map(|operation| {
            format!(
                "{{\"operation_id\":{},\"plan_id\":{},\"root_path\":{},\"completed_count\":{},\"skipped_count\":{},\"failed_count\":{},\"applied_at_unix_seconds\":{}}}",
                json_string(&operation.operation_id),
                json_string(&operation.plan_id),
                json_string(&operation.root_path.to_string_lossy()),
                operation.completed_count,
                operation.skipped_count,
                operation.failed_count,
                operation.applied_at_unix_seconds
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"history_db_path\":{},\"operations\":[{}]}}",
        json_string(&history_db_path.to_string_lossy()),
        items
    )
}

fn render_apply_preview_json(plan: &Plan, validation: &ValidationReport) -> String {
    format!(
        "{{\"approved\":false,\"message\":{},\"plan\":{}}}",
        json_string("rerun with --yes to apply this plan"),
        render_plan_json(plan, validation)
    )
}

fn render_apply_result_json(
    history_db_path: &std::path::Path,
    plan: &Plan,
    validation: &ValidationReport,
    execution: &ExecutionReport,
) -> String {
    let results = execution
        .results
        .iter()
        .map(|result| {
            let (status_code, reason_code, detail) = match &result.status {
                ActionExecutionStatus::Completed => ("completed", None, None),
                ActionExecutionStatus::Skipped { reason_code, detail } => {
                    ("skipped", Some(reason_code.code()), Some(detail.as_str()))
                }
                ActionExecutionStatus::Failed { detail } => ("failed", None, Some(detail.as_str())),
            };
            format!(
                "{{\"action_id\":{},\"source_relative_path\":{},\"destination_relative_path\":{},\"status_code\":{},\"reason_code\":{},\"detail\":{}}}",
                json_string(result.action_id.as_str()),
                json_string(&result.source_relative_path.to_string_lossy()),
                json_string(&result.destination_relative_path.to_string_lossy()),
                json_string(status_code),
                json_option_string(reason_code),
                json_option_string(detail)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"approved\":true,\"history_db_path\":{},\"operation_id\":{},\"plan_id\":{},\"completed_count\":{},\"skipped_count\":{},\"failed_count\":{},\"planning_skip_count\":{},\"validation\":{},\"results\":[{}]}}",
        json_string(&history_db_path.to_string_lossy()),
        json_string(plan.operation_id.as_str()),
        json_string(plan.plan_id.as_str()),
        execution.completed_count(),
        execution.skipped_count(),
        execution.failed_count(),
        plan.skipped_files.len(),
        render_validation_json(validation),
        results
    )
}

fn render_validation_json(validation: &ValidationReport) -> String {
    let invalid = validation
        .invalid_actions
        .iter()
        .map(|action| {
            format!(
                "{{\"action_id\":{},\"source_relative_path\":{},\"reason_code\":{},\"detail\":{}}}",
                json_string(action.action_id.as_str()),
                json_string(&action.source_relative_path.to_string_lossy()),
                json_string(action.reason_code.code()),
                json_string(&action.detail)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"valid_action_count\":{},\"invalid_actions\":[{}]}}",
        validation.valid_actions.len(),
        invalid
    )
}

fn usage() -> String {
    [
        "Usage:",
        "  tidyup scan --root <path> [--format human|json]",
        "  tidyup plan --root <path> [--format human|json]",
        "  tidyup apply --root <path> [--yes] [--format human|json]",
        "  tidyup history --root <path> [--format human|json]",
        "",
        "If --root is omitted, TidyUp uses the current directory.",
        "`scan` and `plan` are read-only.",
        "`apply` asks for confirmation by default and changes files immediately with --yes.",
    ]
    .join("\n")
}

fn json_string(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('\"', "\\\"")
        .replace('\n', "\\n");
    format!("\"{escaped}\"")
}

fn json_option_string(value: Option<&str>) -> String {
    value.map_or_else(|| "null".to_owned(), json_string)
}

#[cfg(test)]
mod tests {
    use super::{run, run_with_prompt};
    use std::io::Cursor;

    #[test]
    fn help_is_returned_without_arguments() {
        let output = run(Vec::new()).expect("usage should render");
        assert!(output.contains("tidyup scan"));
        assert!(output.contains("tidyup apply"));
    }

    #[test]
    fn unknown_command_is_rejected() {
        let error = run(vec!["organize".to_owned()]).expect_err("command should fail");
        assert!(error.contains("unknown command"));
    }

    #[test]
    fn scan_defaults_root_to_current_directory() {
        let original = std::env::current_dir().expect("cwd should exist");
        let temp_dir =
            std::env::temp_dir().join(format!("tidyup-cli-default-root-{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).expect("temp dir should exist");
        std::env::set_current_dir(&temp_dir).expect("should switch cwd");

        let result = run(vec!["scan".to_owned()]);

        std::env::set_current_dir(original).expect("should restore cwd");
        let _ = std::fs::remove_dir_all(&temp_dir);

        let output = result.expect("scan should use cwd");
        assert!(output.contains("Scanned root:"));
    }

    #[test]
    fn apply_can_be_cancelled_interactively() {
        let temp_dir =
            std::env::temp_dir().join(format!("tidyup-cli-apply-cancel-{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).expect("temp dir should exist");
        std::fs::write(temp_dir.join("todo.md"), b"todo").expect("file should exist");

        let mut input = Cursor::new(b"n\n");
        let mut output = Vec::new();
        let result = run_with_prompt(
            vec![
                "apply".to_owned(),
                "--root".to_owned(),
                temp_dir.to_string_lossy().into_owned(),
            ],
            &mut input,
            &mut output,
        )
        .expect("apply should succeed");

        assert_eq!(result, "Apply cancelled. No files were changed.");
        assert!(temp_dir.join("todo.md").exists());
        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
