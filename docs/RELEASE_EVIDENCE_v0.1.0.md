# Release Evidence `v0.1.0`

Recorded on August 19, 2026.

## Local Validation Completed

Commands run successfully in the repository root:

```bash
cargo test --workspace
cargo build --release -p tidyup-cli
```

Validated behavior includes:

- read-only `scan` and `plan`
- interactive and non-interactive `apply`
- SQLite-backed history
- `history show`
- guarded `undo`
- Unicode filenames
- space-containing paths
- link-like entry handling where supported
- blocked undo when the original path is occupied

## Disposable Demo Evidence

Successful demo root:

- root: `/private/tmp/tidyup-demo-pass`
- `todo.md` moved to `Documents/todo.md`
- `photo.jpg` moved to `Images/photo.jpg`
- history database created at `.tidyup/history.sqlite3`

Intentional safety-conflict demo root:

- root: `/private/tmp/tidyup-demo-conflict`
- `report.md` was not moved because `Documents/report.md` already existed
- TidyUp reported the file as left unchanged before execution

## Release-Facing Docs Present

The repository includes:

- `README.md`
- `docs/INSTALLATION.md`
- `docs/PLATFORM_LIMITATIONS.md`
- `docs/RELEASE_NOTES_v0.1.0.md`
- `CHANGELOG.md`
- `RELEASE_CHECKLIST.md`

## Remaining External Evidence

The repository is prepared for GitHub-hosted CI and release artifact generation through:

- `.github/workflows/ci.yml`
- `.github/workflows/release.yml`

For the current MVP presentation, the primary evidence is the macOS local validation and demo flow recorded above.

Windows CI and Windows packaging should be tracked as deferred follow-up work in GitHub issues rather than presented as part of the current MVP release claim.
