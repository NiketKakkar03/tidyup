# Platform Limitations

Known limitations as of August 19, 2026:

- symlink fixture behavior is covered only where the platform supports it
- case-collision layouts are only fully testable on case-sensitive filesystems
- locked or permission-sensitive rename behavior is not perfectly reproducible across macOS and Windows in identical ways
- the CLI warns on project-like folders, but it does not forbid them yet

What is covered today:

- Unicode filenames
- space-containing filenames
- same-root collision behavior
- stale source and occupied destination validation
- blocked undo when the original path is occupied
