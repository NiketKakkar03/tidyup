# Contributing

Thanks for helping with TidyUp.

By participating, you agree to follow the [Code of Conduct](CODE_OF_CONDUCT.md).
Report security vulnerabilities privately by following [SECURITY.md](SECURITY.md)
instead of opening a public issue.

## Principles

- prefer safety over cleverness
- preserve same-root containment
- never add overwrite behavior silently
- keep the MVP narrow
- treat plans, rule packs, and journal contents as untrusted data

## Contributor Setup

Fork `NiketKakkar03/tidyup` on GitHub, then clone your fork. Replace
`YOUR_GITHUB_USERNAME` with your GitHub username:

```bash
git clone https://github.com/YOUR_GITHUB_USERNAME/tidyup.git
cd tidyup
git remote add upstream https://github.com/NiketKakkar03/tidyup.git
git fetch upstream
git switch -c your-focused-branch upstream/main
```

Keep `origin` pointed at your fork and use `upstream` to follow the main
TidyUp repository. Before starting new work, fetch `upstream` and create a new
branch from `upstream/main`.

## Useful Commands

Run `cargo fmt --all` to apply formatting. Before opening a pull request, run:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --release --locked -p tidyup-cli
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

- open an issue first for substantial features or safety-model changes
- keep each pull request focused on one change
- describe the user-facing effect
- mention safety implications explicitly
- include tests for behavior changes
- update docs when commands or workflow changes
- ensure formatting, Clippy, and all workspace tests pass
- push your branch to your fork and open a pull request against
  `NiketKakkar03/tidyup:main`
- use clear commit messages and respond to review feedback with additional
  commits
