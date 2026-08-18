use std::env;
use std::path::PathBuf;

use tidyup_core::{
    RulePackV1, build_plan, render_plan_json, render_scan_json, scan_root, validate_plan,
};

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
    let Some(command) = args.first().map(String::as_str) else {
        return Ok(usage());
    };

    match command {
        "scan" => {
            let options = CommandOptions::parse(&args[1..])?;
            run_scan(options)
        }
        "plan" => {
            let options = CommandOptions::parse(&args[1..])?;
            run_plan(options)
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

struct CommandOptions {
    root: PathBuf,
    format: OutputFormat,
}

impl CommandOptions {
    fn parse(args: &[String]) -> Result<Self, String> {
        let mut root = None;
        let mut format = OutputFormat::Human;
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
                "--help" | "-h" => return Err(usage()),
                other => return Err(format!("unexpected argument: {other}\n\n{}", usage())),
            }
        }

        let root =
            root.ok_or_else(|| format!("missing required --root argument\n\n{}", usage()))?;
        Ok(Self { root, format })
    }
}

fn run_scan(options: CommandOptions) -> Result<String, String> {
    let scan = scan_root(&options.root).map_err(|error| error.to_string())?;

    Ok(match options.format {
        OutputFormat::Human => render_scan_human(&scan),
        OutputFormat::Json => render_scan_json(&scan),
    })
}

fn run_plan(options: CommandOptions) -> Result<String, String> {
    let scan = scan_root(&options.root).map_err(|error| error.to_string())?;
    let plan = build_plan(&scan, &RulePackV1::built_in()).map_err(|error| error.to_string())?;
    let validation = validate_plan(&options.root, &plan);

    Ok(match options.format {
        OutputFormat::Human => render_plan_human(&scan, &plan, &validation),
        OutputFormat::Json => render_plan_json(&plan, &validation),
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

fn render_plan_human(
    scan: &tidyup_core::ScanReport,
    plan: &tidyup_core::Plan,
    validation: &tidyup_core::ValidationReport,
) -> String {
    let mut lines = vec![
        format!("Planned root: {}", scan.root.display()),
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

fn usage() -> String {
    [
        "Usage:",
        "  tidyup scan --root <path> [--format human|json]",
        "  tidyup plan --root <path> [--format human|json]",
        "",
        "Both commands are read-only in this MVP stage.",
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::run;

    #[test]
    fn help_is_returned_without_arguments() {
        let output = run(Vec::new()).expect("usage should render");
        assert!(output.contains("tidyup scan"));
    }

    #[test]
    fn unknown_command_is_rejected() {
        let error = run(vec!["organize".to_owned()]).expect_err("command should fail");
        assert!(error.contains("unknown command"));
    }
}
