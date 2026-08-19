use tidyup_testkit::{FixtureEntry, TestFixture, try_create_symlink_fixture};

fn sample_root_fixture() -> TestFixture {
    let mut entries = vec![
        FixtureEntry::file("Quarterly Notes.txt", b"draft"),
        FixtureEntry::file("todo.md", b"task list"),
        FixtureEntry::file("Photos/IMG_0001.JPG", b"jpeg"),
        FixtureEntry::file("Documents/Quarterly Notes.txt", b"existing collision"),
        FixtureEntry::directory("Empty Folder"),
    ];

    if let Ok(link_entry) =
        try_create_symlink_fixture("Shortcuts/Notes Link", "../Quarterly Notes.txt")
    {
        entries.push(link_entry);
    }

    TestFixture::new(&entries).expect("acceptance fixture should be created")
}

#[test]
fn acceptance_fixture_exercises_spaces_unicode_and_link_like_entries() {
    let fixture = sample_root_fixture();

    assert!(fixture.path("Quarterly Notes.txt").exists());
    assert!(fixture.path("Documents/Quarterly Notes.txt").exists());
    assert!(fixture.path("Empty Folder").is_dir());
}

#[test]
fn scan_command_reports_supported_and_skipped_entries() {
    let fixture = sample_root_fixture();
    let output = run_tidyup_in_root(fixture.root(), ["scan"], None);

    assert!(output.contains("TidyUp Scan"));
    assert!(output.contains("Files TidyUp can consider:"));
    assert!(output.contains("Quarterly Notes.txt"));
    assert!(output.contains("Empty Folder (directory)"));
    assert!(output.contains("Scan is read-only. No files were changed."));
}

#[test]
fn plan_command_reports_moves_skips_and_read_only_guarantee() {
    let fixture = sample_root_fixture();
    let output = run_tidyup_in_root(fixture.root(), ["plan"], None);

    assert!(output.contains("TidyUp Plan"));
    assert!(output.contains("Proposed moves:"));
    assert!(output.contains("todo.md -> Documents/todo.md"));
    assert!(output.contains("destination already exists"));
}

#[test]
fn plan_json_contains_validation_and_reason_codes() {
    let fixture = sample_root_fixture();
    let output = run_tidyup_in_root(fixture.root(), ["plan", "--format", "json"], None);

    assert!(output.contains("\"moves\""));
    assert!(output.contains("\"validation\""));
    assert!(output.contains("\"reason_code\":\"destination_exists\""));
}

#[test]
fn apply_preview_requires_explicit_confirmation() {
    let fixture = sample_root_fixture();
    let output = run_tidyup_in_root(fixture.root(), ["apply"], Some("n\n"));

    assert!(output.contains("No files were changed yet."));
    assert!(output.contains("Apply these moves? [y/N]:"));
    assert!(output.contains("Moves you are about to approve:"));
    assert!(fixture.path("todo.md").exists());
}

#[test]
fn apply_command_moves_files_and_records_history() {
    let fixture = TestFixture::new(&[
        FixtureEntry::file("todo.md", b"task list"),
        FixtureEntry::file("photo.jpg", b"jpeg"),
    ])
    .expect("fixture should be created");

    let output = run_tidyup_in_root(fixture.root(), ["apply", "--yes"], None);

    assert!(output.contains("2 file(s) moved successfully."));
    assert!(output.contains("History saved to:"));
    assert!(!fixture.path("todo.md").exists());
    assert!(fixture.path("Documents/todo.md").exists());
    assert!(fixture.path("Images/photo.jpg").exists());

    let history = run_tidyup_in_root(fixture.root(), ["history"], None);
    assert!(history.contains("1 recorded operation(s)."));
    assert!(history.contains("Use `tidyup history show <operation-id>`"));
}

#[test]
fn apply_command_reports_safety_conflicts_from_existing_destinations() {
    let fixture = sample_root_fixture();
    let output =
        run_tidyup_in_root_allow_exit_code(fixture.root(), ["apply", "--yes"], None, &[0, 2]);

    assert!(output.contains("1 file(s) moved successfully."));
    assert!(output.contains("destination already exists"));
    assert!(fixture.path("Documents/todo.md").exists());
    assert!(fixture.path("Documents/Quarterly Notes.txt").exists());
}

#[test]
fn plan_warns_when_run_in_project_like_folder() {
    let fixture = TestFixture::new(&[
        FixtureEntry::file("Cargo.toml", b"[package]\nname='demo'\n"),
        FixtureEntry::file("todo.md", b"notes"),
        FixtureEntry::directory(".git"),
    ])
    .expect("fixture should be created");

    let output = run_tidyup_in_root(fixture.root(), ["plan"], None);

    assert!(output.contains("Warning: this looks like a project or workspace folder"));
    assert!(output.contains("`Cargo.toml`"));
    assert!(output.contains("`.git`"));
}

#[test]
fn history_show_displays_action_level_details() {
    let fixture = TestFixture::new(&[FixtureEntry::file("todo.md", b"task list")])
        .expect("fixture should be created");
    let _ = run_tidyup_in_root(fixture.root(), ["apply", "--yes"], None);
    let history = run_tidyup_in_root(fixture.root(), ["history"], None);
    let operation_id = history
        .lines()
        .find(|line| line.starts_with("- op-"))
        .and_then(|line| line.split_whitespace().nth(1))
        .expect("operation id should be present");

    let output = run_tidyup_in_root(fixture.root(), ["history", "show", operation_id], None);

    assert!(output.contains("Action results:"));
    assert!(output.contains("todo.md -> Documents/todo.md"));
    assert!(output.contains("completed"));
}

#[test]
fn undo_restores_completed_actions() {
    let fixture = TestFixture::new(&[FixtureEntry::file("todo.md", b"task list")])
        .expect("fixture should be created");
    let _ = run_tidyup_in_root(fixture.root(), ["apply", "--yes"], None);
    let history = run_tidyup_in_root(fixture.root(), ["history"], None);
    let operation_id = history
        .lines()
        .find(|line| line.starts_with("- op-"))
        .and_then(|line| line.split_whitespace().nth(1))
        .expect("operation id should be present")
        .to_owned();

    let output = run_tidyup_in_root(fixture.root(), ["undo", &operation_id], Some("y\n"));

    assert!(output.contains("1 file(s) restored successfully."));
    assert!(fixture.path("todo.md").exists());
    assert!(!fixture.path("Documents/todo.md").exists());
}

#[test]
fn undo_reports_blocked_restore_when_original_path_is_occupied() {
    let fixture = TestFixture::new(&[FixtureEntry::file("todo.md", b"task list")])
        .expect("fixture should be created");
    let _ = run_tidyup_in_root(fixture.root(), ["apply", "--yes"], None);
    std::fs::write(fixture.path("todo.md"), b"new file").expect("original path should be occupied");

    let history = run_tidyup_in_root(fixture.root(), ["history"], None);
    let operation_id = history
        .lines()
        .find(|line| line.starts_with("- op-"))
        .and_then(|line| line.split_whitespace().nth(1))
        .expect("operation id should be present")
        .to_owned();

    let output = run_tidyup_in_root_allow_exit_code(
        fixture.root(),
        ["undo", &operation_id, "--yes"],
        None,
        &[0, 2],
    );

    assert!(output.contains("restore(s) were blocked for safety"));
    assert!(output.contains("original destination is occupied"));
    assert!(fixture.path("todo.md").exists());
    assert!(fixture.path("Documents/todo.md").exists());
}

fn run_tidyup_in_root<const N: usize>(
    root: &std::path::Path,
    args: [&str; N],
    stdin: Option<&str>,
) -> String {
    run_tidyup_in_root_allow_exit_code(root, args, stdin, &[0])
}

fn run_tidyup_in_root_allow_exit_code<const N: usize>(
    root: &std::path::Path,
    args: [&str; N],
    stdin: Option<&str>,
    allowed_exit_codes: &[i32],
) -> String {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_tidyup"))
        .args(args)
        .current_dir(root)
        .stdin(if stdin.is_some() {
            std::process::Stdio::piped()
        } else {
            std::process::Stdio::null()
        })
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            if let Some(stdin_value) = stdin {
                use std::io::Write;
                child
                    .stdin
                    .as_mut()
                    .expect("stdin should be piped")
                    .write_all(stdin_value.as_bytes())?;
            }
            child.wait_with_output()
        })
        .expect("tidyup command should run");

    assert!(
        allowed_exit_codes.contains(&output.status.code().unwrap_or_default()),
        "tidyup exit code {:?} was unexpected, stderr was: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("stdout should be utf-8")
}
