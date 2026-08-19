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

    assert!(output.contains("Read-only scan complete. No files were changed."));
    assert!(output.contains("Quarterly Notes.txt"));
    assert!(output.contains("Empty Folder [directory]"));
    assert!(output.contains("Shortcuts [directory]"));
}

#[test]
fn plan_command_reports_moves_skips_and_read_only_guarantee() {
    let fixture = sample_root_fixture();
    let output = run_tidyup_in_root(fixture.root(), ["plan"], None);

    assert!(output.contains("Read-only plan complete. No files were changed."));
    assert!(output.contains("todo.md -> Documents/todo.md"));
    assert!(output.contains("[destination_exists]"));
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
    assert!(output.contains("Proposed moves:"));
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

    assert!(output.contains("Completed moves: 2"));
    assert!(output.contains("History database:"));
    assert!(!fixture.path("todo.md").exists());
    assert!(fixture.path("Documents/todo.md").exists());
    assert!(fixture.path("Images/photo.jpg").exists());

    let history = run_tidyup_in_root(fixture.root(), ["history"], None);
    assert!(history.contains("Recorded operations: 1"));
}

#[test]
fn apply_command_reports_safety_conflicts_from_existing_destinations() {
    let fixture = sample_root_fixture();
    let output = run_tidyup_in_root(fixture.root(), ["apply", "--yes"], None);

    assert!(output.contains("Completed moves: 1"));
    assert!(output.contains("Planning skips: 1"));
    assert!(output.contains("[destination_exists]"));
    assert!(fixture.path("Documents/todo.md").exists());
    assert!(fixture.path("Documents/Quarterly Notes.txt").exists());
}

fn run_tidyup_in_root<const N: usize>(
    root: &std::path::Path,
    args: [&str; N],
    stdin: Option<&str>,
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
        output.status.success(),
        "tidyup should succeed, stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("stdout should be utf-8")
}
