# CLI Contract

Current user-facing commands:

- `tidyup scan`
- `tidyup plan`
- `tidyup apply`
- `tidyup history`
- `tidyup history show <operation-id>`
- `tidyup undo <operation-id>`

Behavior notes:

- if `--root` is omitted, the current directory is used
- `scan` and `plan` are read-only
- `apply` and `undo` prompt for confirmation unless `--yes` is supplied
- `--format json` is supported on every command

Exit codes:

- `0` success
- `1` usage or execution error
- `2` partial success or blocked safety outcome
