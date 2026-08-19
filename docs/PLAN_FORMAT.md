# Plan Format

Plans are internal typed structures today, with stable JSON output exposed by the CLI.

A plan currently includes:

- schema version
- plan id
- operation id
- selected root
- proposed moves
- planning skips

Each proposed move includes:

- action id
- source snapshot
- destination relative path
- destination directory
- rule id
- human-readable reason

Plans are proposals, not permissions. Every action is revalidated at execution time.
