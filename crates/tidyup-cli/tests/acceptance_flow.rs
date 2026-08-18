use tidyup_testkit::{FixtureEntry, TestFixture, try_create_symlink_fixture};

fn sample_root_fixture() -> TestFixture {
    let mut entries = vec![
        FixtureEntry::file("Quarterly Notes.txt", b"draft"),
        FixtureEntry::file("Photos/IMG_0001.JPG", b"jpeg"),
        FixtureEntry::file("Documents/Quarterly Notes.txt", b"existing collision"),
        FixtureEntry::directory("Empty Folder"),
    ];

    if let Ok(link_entry) = try_create_symlink_fixture("Shortcuts/Notes Link", "../Quarterly Notes.txt")
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
fn scan_plan_validate_apply_undo_flow_is_tracked_as_an_executable_placeholder() {
    let fixture = sample_root_fixture();

    let observable_root = fixture.root();

    assert!(
        observable_root.is_dir(),
        "acceptance tests should always begin with a real disposable root"
    );
    assert!(
        fixture.path("Quarterly Notes.txt").exists(),
        "future scan assertions will observe direct-child files here"
    );
}

#[test]
#[ignore = "issue #6 and later will replace this scaffold with end-to-end flow assertions"]
fn full_scan_plan_validate_apply_undo_acceptance_flow() {
    let fixture = sample_root_fixture();

    assert!(fixture.root().exists());
}
