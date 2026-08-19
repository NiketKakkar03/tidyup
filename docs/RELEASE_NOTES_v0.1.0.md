# Release Notes `v0.1.0`

TidyUp `v0.1.0` is the first narrow MVP release.

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
- no GUI installer
- no automatic collision renaming
