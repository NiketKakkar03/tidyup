# Contributing

Thanks for helping with TidyUp.

## Principles

- prefer safety over cleverness
- preserve same-root containment
- never add overwrite behavior silently
- keep the MVP narrow
- treat plans, rule packs, and journal contents as untrusted data

## Local Setup

```bash
git clone https://github.com/NiketKakkar03/tidyup.git
cd tidyup
cargo build
```

## Useful Commands

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p tidyup-cli -- plan
```

## How To Test Changes Safely

Use disposable folders instead of personal data:

```bash
mkdir -p /tmp/tidyup-scratch
printf "todo" > /tmp/tidyup-scratch/todo.md
cargo run -p tidyup-cli -- apply --root /tmp/tidyup-scratch
```

## Areas Of The Codebase

- `crates/tidyup-core`: planning, validation, execution semantics
- `crates/tidyup-platform`: filesystem primitives
- `crates/tidyup-storage`: SQLite-backed operation journal
- `crates/tidyup-cli`: user-facing CLI
- `crates/tidyup-testkit`: disposable fixtures for acceptance-style tests

## Starter Tasks

Good first contributions:

- improve CLI wording or examples
- add rule-pack documentation examples
- add edge-case tests for fixtures or planning
- improve release/install docs

Suggested issue labels:

- `good first issue`
- `area:cli`
- `area:testing`
- `area:docs`
- `safety`

## Pull Request Expectations

- describe the user-facing effect
- mention safety implications explicitly
- include tests for behavior changes
- update docs when commands or workflow changes
