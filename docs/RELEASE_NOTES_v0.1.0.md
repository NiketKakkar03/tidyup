# Release Notes `v0.1.0`

TidyUp `v0.1.0` is the first narrow macOS MVP release.

Downloads are provided separately for Apple Silicon and Intel Macs. Each archive includes a no-admin installer, an uninstaller, and a SHA-256 checksum.

TidyUp is released under the MIT License. The initial binaries are unsigned, so macOS may require first-run approval in System Settings under Privacy & Security.

What it does:

- previews direct-child file organization inside one selected folder
- applies safe same-root moves
- records operations in SQLite history
- shows prior operations and action-level details
- restores completed actions with guarded undo

Safety guarantees:

- no silent overwrite
- selected-root containment
- execution-time revalidation
- undo that refuses unsafe restoration

Known limitations:

- direct-child files only
- built-in extension rules only
- no recursive cleanup
- terminal-based installer only
- unsigned and not Apple-notarized
- no automatic collision renaming
- Windows distribution is intentionally deferred until later implementation work
