# Journal Format

TidyUp stores history in a SQLite database at:

```text
.tidyup/history.sqlite3
```

Current tables:

- `schema_migrations`
- `operations`
- `action_results`

`operations` stores one row per apply or undo run:

- operation id
- plan id
- selected root path
- applied timestamp
- completed/skipped/failed counts

`action_results` stores one row per action:

- action id
- source relative path
- destination relative path
- status code
- optional reason code
- optional detail

The journal is append-oriented. TidyUp records new operations instead of mutating old ones into a different story.
