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
    let output = run_tidyup([
        "scan",
        "--root",
        fixture
            .root()
            .to_str()
            .expect("fixture path should be utf-8"),
    ]);

    assert!(output.contains("Read-only scan complete. No files were changed."));
    assert!(output.contains("Quarterly Notes.txt"));
    assert!(output.contains("Empty Folder [directory]"));
    assert!(output.contains("Shortcuts [directory]"));
}

#[test]
fn plan_command_reports_moves_skips_and_read_only_guarantee() {
    let fixture = sample_root_fixture();
    let output = run_tidyup([
        "plan",
        "--root",
        fixture
            .root()
            .to_str()
            .expect("fixture path should be utf-8"),
    ]);

    assert!(output.contains("Read-only plan complete. No files were changed."));
    assert!(output.contains("todo.md -> Documents/todo.md"));
    assert!(output.contains("[destination_exists]"));
}

#[test]
fn plan_json_contains_validation_and_reason_codes() {
    let fixture = sample_root_fixture();
    let output = run_tidyup([
        "plan",
        "--root",
        fixture
            .root()
            .to_str()
            .expect("fixture path should be utf-8"),
        "--format",
        "json",
    ]);

    assert!(output.contains("\"moves\""));
    assert!(output.contains("\"validation\""));
    assert!(output.contains("\"reason_code\":\"destination_exists\""));
}

fn run_tidyup<const N: usize>(args: [&str; N]) -> String {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_tidyup"))
        .args(args)
        .output()
        .expect("tidyup command should run");

    assert!(
        output.status.success(),
        "tidyup should succeed, stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("stdout should be utf-8")
}
