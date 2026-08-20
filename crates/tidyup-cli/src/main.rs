use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use tidyup_core::{
    ActionExecutionStatus, ActionId, ExecutionReport, FileSnapshot, PLAN_SCHEMA_VERSION, Plan,
    PlanSkip, PlannedMove, RulePackV1, ValidationReport, build_plan, execute_plan,
    render_plan_json, render_scan_json, scan_root, validate_plan,
};
use tidyup_storage::{
    ActionResultRecord, OperationRecord, default_history_db_path, load_action_results,
    load_operations, persist_execution,
};

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    match run(&args) {
        Ok(outcome) => {
            if !outcome.output.is_empty() {
                println!("{}", outcome.output);
            }
            std::process::exit(outcome.exit_code);
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

#[derive(Debug)]
struct CommandOutcome {
    output: String,
    exit_code: i32,
}

impl CommandOutcome {
    fn success(output: String) -> Self {
        Self {
            output,
            exit_code: 0,
        }
    }

    fn partial(output: String) -> Self {
        Self {
            output,
            exit_code: 2,
        }
    }
}

fn run(args: &[String]) -> Result<CommandOutcome, String> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    run_with_prompt(args, &mut stdin.lock(), &mut stdout.lock())
}

fn run_with_prompt(
    args: &[String],
    input: &mut impl io::BufRead,
    output: &mut impl Write,
) -> Result<CommandOutcome, String> {
    let Some(command) = args.first().map(String::as_str) else {
        return Ok(CommandOutcome::success(usage()));
    };

    match command {
        "scan" => run_scan(&ReadOnlyCommandOptions::parse(&args[1..])?),
        "plan" => run_plan(&ReadOnlyCommandOptions::parse(&args[1..])?),
        "apply" => run_apply(&ApplyCommandOptions::parse(&args[1..])?, input, output),
        "history" => run_history(HistoryCommand::parse(&args[1..])?),
        "undo" => run_undo(&UndoCommandOptions::parse(&args[1..])?, input, output),
        "--help" | "-h" | "help" => Ok(CommandOutcome::success(usage())),
        _ => Err(format!("unknown command: {command}\n\n{}", usage())),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutputFormat {
    Human,
    Json,
}

#[derive(Clone, Debug)]
struct CommonOptions {
    root: PathBuf,
    format: OutputFormat,
    approve: bool,
    verbose: bool,
}

impl CommonOptions {
    fn parse(args: &[String]) -> Result<Self, String> {
        let mut root = None;
        let mut format = OutputFormat::Human;
        let mut approve = false;
        let mut verbose = false;
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
                "--verbose" => {
                    verbose = true;
                    index += 1;
                }
                "--help" | "-h" => return Err(usage()),
                other => return Err(format!("unexpected argument: {other}\n\n{}", usage())),
            }
        }

        Ok(Self {
            root: root.unwrap_or(env::current_dir().map_err(|error| error.to_string())?),
            format,
            approve,
            verbose,
        })
    }
}

struct ReadOnlyCommandOptions {
    common: CommonOptions,
}

impl ReadOnlyCommandOptions {
    fn parse(args: &[String]) -> Result<Self, String> {
        Ok(Self {
            common: CommonOptions::parse(args)?,
        })
    }
}

struct ApplyCommandOptions {
    common: CommonOptions,
}

impl ApplyCommandOptions {
    fn parse(args: &[String]) -> Result<Self, String> {
        Ok(Self {
            common: CommonOptions::parse(args)?,
        })
    }
}

enum HistoryCommand {
    List(CommonOptions),
    Show {
        options: CommonOptions,
        operation_id: String,
    },
}

impl HistoryCommand {
    fn parse(args: &[String]) -> Result<Self, String> {
        if args.first().map(String::as_str) == Some("show") {
            let operation_id = args
                .get(1)
                .ok_or_else(|| "tidyup history show requires an operation id".to_owned())?
                .clone();
            let options = CommonOptions::parse(&args[2..])?;
            Ok(Self::Show {
                options,
                operation_id,
            })
        } else {
            Ok(Self::List(CommonOptions::parse(args)?))
        }
    }
}

struct UndoCommandOptions {
    common: CommonOptions,
    operation_id: String,
}

impl UndoCommandOptions {
    fn parse(args: &[String]) -> Result<Self, String> {
        let operation_id = args
            .first()
            .ok_or_else(|| "tidyup undo requires an operation id".to_owned())?
            .clone();
        Ok(Self {
            common: CommonOptions::parse(&args[1..])?,
            operation_id,
        })
    }
}

fn run_scan(options: &ReadOnlyCommandOptions) -> Result<CommandOutcome, String> {
    let scan = scan_root(&options.common.root).map_err(|error| error.to_string())?;
    Ok(CommandOutcome::success(match options.common.format {
        OutputFormat::Human => render_scan_human(&scan),
        OutputFormat::Json => render_scan_json(&scan),
    }))
}

fn run_plan(options: &ReadOnlyCommandOptions) -> Result<CommandOutcome, String> {
    let scan = scan_root(&options.common.root).map_err(|error| error.to_string())?;
    let plan = build_plan(&scan, &RulePackV1::built_in()).map_err(|error| error.to_string())?;
    let validation = validate_plan(&options.common.root, &plan);

    Ok(CommandOutcome::success(match options.common.format {
        OutputFormat::Human => render_plan_human(&plan, &validation),
        OutputFormat::Json => render_plan_json(&plan, &validation),
    }))
}

fn run_apply(
    options: &ApplyCommandOptions,
    input: &mut impl io::BufRead,
    output: &mut impl Write,
) -> Result<CommandOutcome, String> {
    let scan = scan_root(&options.common.root).map_err(|error| error.to_string())?;
    let plan = build_plan(&scan, &RulePackV1::built_in()).map_err(|error| error.to_string())?;
    let validation = validate_plan(&options.common.root, &plan);

    if !options.common.approve {
        if options.common.format == OutputFormat::Json {
            return Ok(CommandOutcome::success(render_apply_preview_json(
                "apply",
                &plan,
                &validation,
            )));
        }

        let preview = render_apply_preview_human(&plan, &validation, options.common.verbose);
        writeln!(output, "{preview}").map_err(|error| error.to_string())?;
        write!(output, "Apply these moves? [y/N]: ").map_err(|error| error.to_string())?;
        output.flush().map_err(|error| error.to_string())?;

        let mut response = String::new();
        input
            .read_line(&mut response)
            .map_err(|error| error.to_string())?;
        if !matches!(response.trim(), "y" | "Y" | "yes" | "YES" | "Yes") {
            return Ok(CommandOutcome::success(
                "Apply cancelled. No files were changed.".to_owned(),
            ));
        }
    }

    let execution = execute_plan(&options.common.root, &plan);
    let history_db_path = default_history_db_path(&options.common.root);
    persist_execution(&history_db_path, &plan, &execution).map_err(|error| error.to_string())?;

    let output_text = match options.common.format {
        OutputFormat::Human => {
            render_apply_result_human(&history_db_path, &plan, &execution, options.common.verbose)
        }
        OutputFormat::Json => render_apply_result_json(&history_db_path, &plan, &execution),
    };

    Ok(
        if execution.skipped_count() > 0 || execution.failed_count() > 0 {
            CommandOutcome::partial(output_text)
        } else {
            CommandOutcome::success(output_text)
        },
    )
}

fn run_history(command: HistoryCommand) -> Result<CommandOutcome, String> {
    match command {
        HistoryCommand::List(options) => {
            let history_db_path = default_history_db_path(&options.root);
            let operations =
                load_operations(&history_db_path).map_err(|error| error.to_string())?;
            Ok(CommandOutcome::success(match options.format {
                OutputFormat::Human => render_history_human(&history_db_path, &operations),
                OutputFormat::Json => render_history_json(&history_db_path, &operations),
            }))
        }
        HistoryCommand::Show {
            options,
            operation_id,
        } => {
            let history_db_path = default_history_db_path(&options.root);
            let operations =
                load_operations(&history_db_path).map_err(|error| error.to_string())?;
            let operation = find_operation(&operations, &operation_id)?;
            let action_results = load_action_results(&history_db_path, &operation_id)
                .map_err(|error| error.to_string())?;
            Ok(CommandOutcome::success(match options.format {
                OutputFormat::Human => {
                    render_history_show_human(&history_db_path, operation, &action_results)
                }
                OutputFormat::Json => {
                    render_history_show_json(&history_db_path, operation, &action_results)
                }
            }))
        }
    }
}

fn run_undo(
    options: &UndoCommandOptions,
    input: &mut impl io::BufRead,
    output: &mut impl Write,
) -> Result<CommandOutcome, String> {
    let history_db_path = default_history_db_path(&options.common.root);
    let operations = load_operations(&history_db_path).map_err(|error| error.to_string())?;
    let operation = find_operation(&operations, &options.operation_id)?;
    let action_results = load_action_results(&history_db_path, &options.operation_id)
        .map_err(|error| error.to_string())?;

    let undo_plan = build_undo_plan(&options.common.root, &options.operation_id, &action_results)?;
    let validation = validate_plan(&options.common.root, &undo_plan);

    if !options.common.approve {
        if options.common.format == OutputFormat::Json {
            return Ok(CommandOutcome::success(render_apply_preview_json(
                "undo",
                &undo_plan,
                &validation,
            )));
        }

        let preview =
            render_undo_preview_human(operation, &undo_plan, &validation, options.common.verbose);
        writeln!(output, "{preview}").map_err(|error| error.to_string())?;
        write!(output, "Restore these files? [y/N]: ").map_err(|error| error.to_string())?;
        output.flush().map_err(|error| error.to_string())?;

        let mut response = String::new();
        input
            .read_line(&mut response)
            .map_err(|error| error.to_string())?;
        if !matches!(response.trim(), "y" | "Y" | "yes" | "YES" | "Yes") {
            return Ok(CommandOutcome::success(
                "Undo cancelled. No files were changed.".to_owned(),
            ));
        }
    }

    let execution = execute_plan(&options.common.root, &undo_plan);
    persist_execution(&history_db_path, &undo_plan, &execution)
        .map_err(|error| error.to_string())?;

    let output_text = match options.common.format {
        OutputFormat::Human => render_undo_result_human(
            &history_db_path,
            operation,
            &undo_plan,
            &execution,
            options.common.verbose,
        ),
        OutputFormat::Json => render_apply_result_json(&history_db_path, &undo_plan, &execution),
    };

    Ok(
        if execution.completed_count() == 0
            || execution.skipped_count() > 0
            || execution.failed_count() > 0
        {
            CommandOutcome::partial(output_text)
        } else {
            CommandOutcome::success(output_text)
        },
    )
}

fn build_undo_plan(
    root: &Path,
    target_operation_id: &str,
    action_results: &[ActionResultRecord],
) -> Result<Plan, String> {
    let mut moves = Vec::new();

    for result in action_results
        .iter()
        .filter(|record| record.status_code == "completed")
    {
        let current_source = root.join(&result.destination_relative_path);
        let source_snapshot = FileSnapshot::from_root(root, &current_source).map_err(|error| {
            format!("could not prepare undo plan for operation {target_operation_id}: {error}")
        })?;
        let destination_dir = result
            .source_relative_path
            .parent()
            .map(|parent| parent.to_string_lossy().into_owned())
            .unwrap_or_default();

        moves.push(PlannedMove {
            action_id: ActionId::from_index(moves.len() + 1),
            source: source_snapshot,
            destination_dir,
            destination_relative_path: result.source_relative_path.clone(),
            rule_id: "undo".to_owned(),
            reason: format!(
                "restore {} from operation {}",
                result.source_relative_path.display(),
                target_operation_id
            ),
        });
    }

    if moves.is_empty() {
        return Err(format!(
            "operation {target_operation_id} has no completed actions to undo"
        ));
    }

    Ok(Plan {
        schema_version: PLAN_SCHEMA_VERSION.to_owned(),
        plan_id: tidyup_core::PlanId::new(),
        operation_id: tidyup_core::OperationId::new(),
        root: root.to_path_buf(),
        moves,
        skipped_files: Vec::<PlanSkip>::new(),
    })
}

fn render_scan_human(scan: &tidyup_core::ScanReport) -> String {
    let mut lines = banner_lines("Scan", &scan.root);
    lines.push(format!(
        "{} direct-child files can be evaluated.",
        scan.scanned_files.len()
    ));
    if !scan.skipped_entries.is_empty() {
        lines.push(format!(
            "{} entries will be left unchanged because they are directories or risky items.",
            scan.skipped_entries.len()
        ));
    }

    append_project_like_warning(&mut lines, &scan.root);

    if !scan.scanned_files.is_empty() {
        lines.push("Files TidyUp can consider:".to_owned());
        for file in &scan.scanned_files {
            lines.push(format!(
                "1. {} ({} bytes, {})",
                file.relative_path.display(),
                file.size_bytes,
                file.extension.as_deref().unwrap_or("no extension")
            ));
        }
    }

    if !scan.skipped_entries.is_empty() {
        lines.push("Left unchanged during scan:".to_owned());
        for entry in &scan.skipped_entries {
            lines.push(format!(
                "- {} ({})",
                entry.relative_path.display(),
                human_scan_reason(entry.reason.code())
            ));
        }
    }

    lines.push("Scan is read-only. No files were changed.".to_owned());
    lines.join("\n")
}

fn render_plan_human(plan: &Plan, validation: &ValidationReport) -> String {
    let mut lines = banner_lines("Plan", &plan.root);
    lines.push(format!(
        "{} file(s) can be organized right now.",
        validation.valid_actions.len()
    ));
    lines.push(format!(
        "{} file(s) will stay where they are due to rules or safety checks.",
        plan.skipped_files.len() + validation.invalid_actions.len()
    ));
    append_project_like_warning(&mut lines, &plan.root);

    if !plan.moves.is_empty() {
        lines.push("Proposed moves:".to_owned());
        for (index, planned_move) in plan.moves.iter().enumerate() {
            lines.push(format!(
                "{}. {} -> {}",
                index + 1,
                planned_move.source.relative_path.display(),
                planned_move.destination_relative_path.display()
            ));
        }
    }

    append_plan_skip_sections(&mut lines, plan, validation);
    lines.push("Plan is read-only. No files were changed.".to_owned());
    lines.join("\n")
}

fn render_apply_preview_human(plan: &Plan, validation: &ValidationReport, verbose: bool) -> String {
    let mut lines = banner_lines("Apply Preview", &plan.root);
    lines.push(format!(
        "{} file(s) are ready to move now.",
        validation.valid_actions.len()
    ));
    lines.push(format!(
        "{} file(s) will be left unchanged.",
        plan.skipped_files.len() + validation.invalid_actions.len()
    ));
    append_project_like_warning(&mut lines, &plan.root);

    if !validation.valid_actions.is_empty() {
        lines.push("Moves you are about to approve:".to_owned());
        for (index, planned_move) in validation.valid_actions.iter().enumerate() {
            lines.push(render_move_line(
                &plan.root,
                index + 1,
                &planned_move.source.relative_path,
                &planned_move.destination_relative_path,
                verbose,
            ));
        }
    }

    append_plan_skip_sections(&mut lines, plan, validation);
    lines.push("No files were changed yet.".to_owned());
    lines.join("\n")
}

fn render_apply_result_human(
    history_db_path: &Path,
    plan: &Plan,
    execution: &ExecutionReport,
    verbose: bool,
) -> String {
    let mut lines = banner_lines("Apply Result", &plan.root);
    lines.push(format!(
        "{} file(s) moved successfully.",
        execution.completed_count()
    ));
    if execution.skipped_count() > 0 {
        lines.push(format!(
            "{} file(s) were skipped at execution time.",
            execution.skipped_count()
        ));
    }
    if execution.failed_count() > 0 {
        lines.push(format!(
            "{} file(s) failed during execution.",
            execution.failed_count()
        ));
    }
    lines.push(format!("Operation id: {}", plan.operation_id.as_str()));
    lines.push(format!("History saved to: {}", history_db_path.display()));
    lines.push("Apply results:".to_owned());
    for result in &execution.results {
        lines.push(render_result_line(
            &plan.root,
            &result.source_relative_path,
            &result.destination_relative_path,
            &human_execution_status(&result.status),
            verbose,
        ));
    }
    if !plan.skipped_files.is_empty() {
        lines.push("Files left unchanged before execution:".to_owned());
        for skip in &plan.skipped_files {
            lines.push(format!(
                "- {} ({})",
                skip.source_relative_path.display(),
                human_plan_reason(skip.reason_code.code())
            ));
        }
    }
    lines.join("\n")
}

fn render_history_human(history_db_path: &Path, operations: &[OperationRecord]) -> String {
    let mut lines = vec![
        "TidyUp History".to_owned(),
        format!("Database: {}", history_db_path.display()),
        format!("{} recorded operation(s).", operations.len()),
    ];

    if operations.is_empty() {
        lines.push("No operations have been recorded for this folder yet.".to_owned());
        return lines.join("\n");
    }

    lines.push("Recent operations:".to_owned());
    for operation in operations {
        lines.push(format!(
            "- {} completed={} skipped={} failed={}",
            operation.operation_id,
            operation.completed_count,
            operation.skipped_count,
            operation.failed_count
        ));
    }
    lines.push("Use `tidyup history show <operation-id>` to inspect one operation.".to_owned());
    lines.join("\n")
}

fn render_history_show_human(
    history_db_path: &Path,
    operation: &OperationRecord,
    action_results: &[ActionResultRecord],
) -> String {
    let mut lines = vec![
        format!("Operation {}", operation.operation_id),
        format!("Database: {}", history_db_path.display()),
        format!("Root: {}", operation.root_path.display()),
        format!(
            "completed={} skipped={} failed={}",
            operation.completed_count, operation.skipped_count, operation.failed_count
        ),
    ];
    lines.push("Action results:".to_owned());
    for result in action_results {
        lines.push(format!(
            "- {} -> {} ({})",
            result.source_relative_path.display(),
            result.destination_relative_path.display(),
            human_action_record_status(result)
        ));
    }
    lines.join("\n")
}

fn render_undo_preview_human(
    operation: &OperationRecord,
    plan: &Plan,
    validation: &ValidationReport,
    verbose: bool,
) -> String {
    let mut lines = banner_lines("Undo Preview", &plan.root);
    lines.push(format!(
        "Preparing to restore files from operation {}.",
        operation.operation_id
    ));
    lines.push(format!(
        "{} file(s) can be restored right now.",
        validation.valid_actions.len()
    ));
    if !validation.invalid_actions.is_empty() {
        lines.push(format!(
            "{} file(s) cannot be restored safely yet.",
            validation.invalid_actions.len()
        ));
    }

    lines.push("Planned restores:".to_owned());
    for (index, planned_move) in plan.moves.iter().enumerate() {
        lines.push(render_move_line(
            &plan.root,
            index + 1,
            &planned_move.source.relative_path,
            &planned_move.destination_relative_path,
            verbose,
        ));
    }
    if !validation.invalid_actions.is_empty() {
        lines.push("Blocked restores:".to_owned());
        for action in &validation.invalid_actions {
            lines.push(format!(
                "- {} ({})",
                action.source_relative_path.display(),
                human_validation_reason(action.reason_code.code())
            ));
        }
    }
    lines.push("No files were changed yet.".to_owned());
    lines.join("\n")
}

fn render_undo_result_human(
    history_db_path: &Path,
    operation: &OperationRecord,
    plan: &Plan,
    execution: &ExecutionReport,
    verbose: bool,
) -> String {
    let mut lines = banner_lines("Undo Result", &plan.root);
    lines.push(format!(
        "Undo attempt for operation {} recorded as {}.",
        operation.operation_id,
        plan.operation_id.as_str()
    ));
    lines.push(format!(
        "{} file(s) restored successfully.",
        execution.completed_count()
    ));
    if execution.skipped_count() > 0 {
        lines.push(format!(
            "{} restore(s) were blocked for safety.",
            execution.skipped_count()
        ));
    }
    if execution.failed_count() > 0 {
        lines.push(format!(
            "{} restore(s) failed during execution.",
            execution.failed_count()
        ));
    }
    lines.push(format!("History saved to: {}", history_db_path.display()));
    lines.push("Undo results:".to_owned());
    for result in &execution.results {
        lines.push(render_result_line(
            &plan.root,
            &result.source_relative_path,
            &result.destination_relative_path,
            &human_execution_status(&result.status),
            verbose,
        ));
    }
    lines.join("\n")
}

fn render_move_line(
    root: &Path,
    index: usize,
    source_relative_path: &Path,
    destination_relative_path: &Path,
    verbose: bool,
) -> String {
    if verbose {
        format!(
            "{}. {} -> {}",
            index,
            root.join(source_relative_path).display(),
            root.join(destination_relative_path).display()
        )
    } else {
        format!(
            "{}. {} -> {}",
            index,
            source_relative_path.display(),
            destination_relative_path.display()
        )
    }
}

fn render_result_line(
    root: &Path,
    source_relative_path: &Path,
    destination_relative_path: &Path,
    status: &str,
    verbose: bool,
) -> String {
    if verbose {
        format!(
            "- {} -> {} ({})",
            root.join(source_relative_path).display(),
            root.join(destination_relative_path).display(),
            status
        )
    } else {
        format!(
            "- {} -> {} ({})",
            source_relative_path.display(),
            destination_relative_path.display(),
            status
        )
    }
}

fn render_history_json(history_db_path: &Path, operations: &[OperationRecord]) -> String {
    let items = operations
        .iter()
        .map(operation_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"history_db_path\":{},\"operations\":[{}]}}",
        json_string(&history_db_path.to_string_lossy()),
        items
    )
}

fn render_history_show_json(
    history_db_path: &Path,
    operation: &OperationRecord,
    action_results: &[ActionResultRecord],
) -> String {
    let items = action_results
        .iter()
        .map(action_record_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"history_db_path\":{},\"operation\":{},\"action_results\":[{}]}}",
        json_string(&history_db_path.to_string_lossy()),
        operation_json(operation),
        items
    )
}

fn render_apply_preview_json(kind: &str, plan: &Plan, validation: &ValidationReport) -> String {
    format!(
        "{{\"approved\":false,\"kind\":{},\"message\":{},\"plan\":{}}}",
        json_string(kind),
        json_string("rerun with confirmation to continue"),
        render_plan_json(plan, validation)
    )
}

fn render_apply_result_json(
    history_db_path: &Path,
    plan: &Plan,
    execution: &ExecutionReport,
) -> String {
    let results = execution
        .results
        .iter()
        .map(execution_result_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"approved\":true,\"history_db_path\":{},\"operation_id\":{},\"plan_id\":{},\"completed_count\":{},\"skipped_count\":{},\"failed_count\":{},\"results\":[{}]}}",
        json_string(&history_db_path.to_string_lossy()),
        json_string(plan.operation_id.as_str()),
        json_string(plan.plan_id.as_str()),
        execution.completed_count(),
        execution.skipped_count(),
        execution.failed_count(),
        results
    )
}

fn usage() -> String {
    [
        "Usage:",
        "  tidyup scan [--root <path>] [--format human|json]",
        "  tidyup plan [--root <path>] [--format human|json]",
        "  tidyup apply [--root <path>] [--yes] [--verbose] [--format human|json]",
        "  tidyup history [--root <path>] [--format human|json]",
        "  tidyup history show <operation-id> [--root <path>] [--format human|json]",
        "  tidyup undo <operation-id> [--root <path>] [--yes] [--format human|json]",
        "",
        "If --root is omitted, TidyUp uses the current directory.",
        "`scan` and `plan` are read-only.",
        "`apply` and `undo` ask for confirmation by default and use --yes for non-interactive runs.",
        "`apply --verbose` and `undo --verbose` show full source and destination paths.",
    ]
    .join("\n")
}

fn banner_lines(title: &str, root: &Path) -> Vec<String> {
    vec![
        format!("TidyUp {title}"),
        format!("Folder: {}", root.display()),
    ]
}

fn append_project_like_warning(lines: &mut Vec<String>, root: &Path) {
    let indicators = project_like_indicators(root);
    if indicators.is_empty() {
        return;
    }

    lines.push(format!(
        "Warning: this looks like a project or workspace folder because it contains {}.",
        indicators
            .iter()
            .map(|value| format!("`{value}`"))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    lines.push(
        "TidyUp is better suited for personal folders like Downloads, Desktop, or exports."
            .to_owned(),
    );
}

fn append_plan_skip_sections(lines: &mut Vec<String>, plan: &Plan, validation: &ValidationReport) {
    let unmatched = plan
        .skipped_files
        .iter()
        .filter(|skip| skip.reason_code == tidyup_core::PlanSkipReason::NoMatchingRule)
        .collect::<Vec<_>>();
    let safety_skips = plan
        .skipped_files
        .iter()
        .filter(|skip| skip.reason_code != tidyup_core::PlanSkipReason::NoMatchingRule)
        .collect::<Vec<_>>();

    if !unmatched.is_empty() {
        lines.push("Left unchanged because no built-in rule matched:".to_owned());
        for skip in unmatched {
            lines.push(format!("- {}", skip.source_relative_path.display()));
        }
    }

    if !safety_skips.is_empty() {
        lines.push("Left unchanged for safety:".to_owned());
        for skip in safety_skips {
            lines.push(format!(
                "- {} ({})",
                skip.source_relative_path.display(),
                human_plan_reason(skip.reason_code.code())
            ));
        }
    }

    if !validation.invalid_actions.is_empty() {
        lines.push("Not safe to run right now:".to_owned());
        for action in &validation.invalid_actions {
            lines.push(format!(
                "- {} ({})",
                action.source_relative_path.display(),
                human_validation_reason(action.reason_code.code())
            ));
        }
    }
}

fn human_scan_reason(code: &str) -> &'static str {
    match code {
        "directory" => "directory",
        "symlink" => "link-like entry",
        "non_file" => "unsupported special file",
        "unsupported_name" => "unsupported name",
        "metadata_read_failed" => "metadata could not be read",
        _ => "unknown reason",
    }
}

fn human_plan_reason(code: &str) -> &'static str {
    match code {
        "destination_exists" => "destination already exists",
        "duplicate_destination" => "another file wants the same destination",
        "invalid_destination_dir" => "destination folder is not safe",
        "no_matching_rule" => "no built-in rule matched",
        _ => "safety check blocked this action",
    }
}

fn human_validation_reason(code: &str) -> &'static str {
    match code {
        "source_missing" => "source file is missing",
        "source_changed" => "source file changed after planning",
        "destination_exists" => "original destination is occupied",
        "invalid_destination" => "destination path is not safe",
        "duplicate_destination" => "another action targets the same path",
        _ => "validation blocked this action",
    }
}

fn human_execution_status(status: &ActionExecutionStatus) -> String {
    match status {
        ActionExecutionStatus::Completed => "completed".to_owned(),
        ActionExecutionStatus::Skipped {
            reason_code,
            detail: _,
        } => format!("skipped: {}", human_validation_reason(reason_code.code())),
        ActionExecutionStatus::Failed { detail } => format!("failed: {detail}"),
    }
}

fn human_action_record_status(record: &ActionResultRecord) -> String {
    match record.status_code.as_str() {
        "completed" => "completed".to_owned(),
        "skipped" => record
            .reason_code
            .as_deref()
            .map(human_validation_reason)
            .map_or_else(|| "skipped".to_owned(), ToOwned::to_owned),
        "failed" => record.detail.clone().unwrap_or_else(|| "failed".to_owned()),
        other => other.to_owned(),
    }
}

fn project_like_indicators(root: &Path) -> Vec<&'static str> {
    let mut indicators = Vec::new();
    for candidate in [
        ".git",
        "Cargo.toml",
        "package.json",
        "pyproject.toml",
        "Makefile",
        ".github",
    ] {
        if root.join(candidate).exists() {
            indicators.push(candidate);
        }
    }
    indicators
}

fn find_operation<'a>(
    operations: &'a [OperationRecord],
    operation_id: &str,
) -> Result<&'a OperationRecord, String> {
    operations
        .iter()
        .find(|operation| operation.operation_id == operation_id)
        .ok_or_else(|| format!("operation {operation_id} was not found in this folder history"))
}

fn operation_json(operation: &OperationRecord) -> String {
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
}

fn action_record_json(record: &ActionResultRecord) -> String {
    format!(
        "{{\"operation_id\":{},\"action_id\":{},\"source_relative_path\":{},\"destination_relative_path\":{},\"status_code\":{},\"reason_code\":{},\"detail\":{}}}",
        json_string(&record.operation_id),
        json_string(&record.action_id),
        json_string(&record.source_relative_path.to_string_lossy()),
        json_string(&record.destination_relative_path.to_string_lossy()),
        json_string(&record.status_code),
        json_option_string(record.reason_code.as_deref()),
        json_option_string(record.detail.as_deref())
    )
}

fn execution_result_json(result: &tidyup_core::ActionExecutionResult) -> String {
    let (status_code, reason_code, detail) = match &result.status {
        ActionExecutionStatus::Completed => ("completed", None, None),
        ActionExecutionStatus::Skipped {
            reason_code,
            detail,
        } => ("skipped", Some(reason_code.code()), Some(detail.as_str())),
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
        let args = Vec::new();
        let output = run(&args).expect("usage should render");
        assert!(output.output.contains("tidyup scan"));
        assert!(output.output.contains("tidyup undo"));
    }

    #[test]
    fn unknown_command_is_rejected() {
        let args = vec!["organize".to_owned()];
        let error = run(&args).expect_err("command should fail");
        assert!(error.contains("unknown command"));
    }

    #[test]
    fn scan_defaults_root_to_current_directory() {
        let original = std::env::current_dir().expect("cwd should exist");
        let temp_dir =
            std::env::temp_dir().join(format!("tidyup-cli-default-root-{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).expect("temp dir should exist");
        std::env::set_current_dir(&temp_dir).expect("should switch cwd");

        let args = vec!["scan".to_owned()];
        let result = run(&args);

        std::env::set_current_dir(original).expect("should restore cwd");
        let _ = std::fs::remove_dir_all(&temp_dir);

        let outcome = result.expect("scan should use cwd");
        assert!(outcome.output.contains("Folder:"));
    }

    #[test]
    fn apply_can_be_cancelled_interactively() {
        let temp_dir =
            std::env::temp_dir().join(format!("tidyup-cli-apply-cancel-{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).expect("temp dir should exist");
        std::fs::write(temp_dir.join("todo.md"), b"todo").expect("file should exist");

        let mut input = Cursor::new(b"n\n");
        let mut output = Vec::new();
        let args = vec![
            "apply".to_owned(),
            "--root".to_owned(),
            temp_dir.to_string_lossy().into_owned(),
        ];
        let result = run_with_prompt(&args, &mut input, &mut output).expect("apply should succeed");

        assert_eq!(result.output, "Apply cancelled. No files were changed.");
        assert!(temp_dir.join("todo.md").exists());
        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
