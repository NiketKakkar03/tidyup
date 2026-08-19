# Architecture

TidyUp is split into focused crates:

- `tidyup-core`
  read-only scan models, rule evaluation, planning, validation, execution semantics, and undo-plan assembly inputs

- `tidyup-platform`
  platform-aware filesystem primitives for safe same-root moves

- `tidyup-storage`
  SQLite-backed operation journal and history queries

- `tidyup-cli`
  user-facing terminal flow, summaries, confirmation prompts, and JSON output

- `tidyup-testkit`
  disposable filesystem fixtures used by unit and acceptance tests

The intended engine flow is:

```text
scan -> classify -> plan -> validate -> apply -> journal -> history -> undo
```

The CLI is only one shell around that engine. The core crate is intentionally shaped so later interfaces can reuse it.
