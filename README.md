# TidyUp

TidyUp is a local-first file organization CLI for messy personal folders.

The MVP promise is narrow on purpose:

- organize one selected folder safely
- preview before mutation
- never overwrite existing files silently
- record what happened
- undo safely when restoration is still valid

TidyUp is not a generic cleanup script and it is not meant for source-code repositories, sync folders, or broad recursive reorganization.

## Current Status

As of August 19, 2026, the `v0.1.0` MVP flow is implemented in the CLI:

```text
scan -> plan -> apply -> history -> history show -> undo
```

The scope is still intentionally limited:

- only direct-child files of the selected root are considered
- moves stay inside the selected root
- built-in classification is extension-based
- no deletion
- no recursive tree cleanup
- no cloud processing
- no GUI

## Quickstart

If you already have Rust installed:

```bash
git clone https://github.com/NiketKakkar03/tidyup.git
cd tidyup
cargo run -p tidyup-cli -- scan
```

TidyUp uses the current directory by default. Use `--root /path/to/folder` when you want to target a different folder.

Suggested first-time workflow:

```bash
cargo run -p tidyup-cli -- scan
cargo run -p tidyup-cli -- plan
cargo run -p tidyup-cli -- apply
cargo run -p tidyup-cli -- history
```

If the preview looks right, confirm the prompt or use `--yes` for non-interactive runs.

## Commands

### `scan`

Read-only. Shows which direct-child files TidyUp can evaluate and which entries will be left alone.

```bash
cargo run -p tidyup-cli -- scan
cargo run -p tidyup-cli -- scan --root ~/Downloads
```

### `plan`

Read-only. Shows proposed moves, files with no matching rule, and safety blocks.

```bash
cargo run -p tidyup-cli -- plan
```

### `apply`

Previews the changes, asks for confirmation, then performs safe same-root moves and records the operation in `.tidyup/history.sqlite3`.

```bash
cargo run -p tidyup-cli -- apply
cargo run -p tidyup-cli -- apply --yes
```

### `history`

Lists recorded operations for the selected folder.

```bash
cargo run -p tidyup-cli -- history
```

### `history show`

Displays action-level details for a single operation.

```bash
cargo run -p tidyup-cli -- history show <operation-id>
```

### `undo`

Previews and safely restores completed actions from a prior operation when restoration is still valid.

```bash
cargo run -p tidyup-cli -- undo <operation-id>
cargo run -p tidyup-cli -- undo <operation-id> --yes
```

## Example Demo

Create a disposable demo folder:

```bash
mkdir -p /tmp/tidyup-demo
printf "todo" > /tmp/tidyup-demo/todo.md
printf "jpeg" > /tmp/tidyup-demo/photo.jpg
```

Run the flow:

```bash
cargo run -p tidyup-cli -- scan --root /tmp/tidyup-demo
cargo run -p tidyup-cli -- plan --root /tmp/tidyup-demo
cargo run -p tidyup-cli -- apply --root /tmp/tidyup-demo
cargo run -p tidyup-cli -- history --root /tmp/tidyup-demo
```

Expected result:

- `todo.md` moves to `Documents/todo.md`
- `photo.jpg` moves to `Images/photo.jpg`
- the operation is recorded in `.tidyup/history.sqlite3`

## JSON Output

All user-facing commands support `--format json`.

Examples:

```bash
cargo run -p tidyup-cli -- plan --format json
cargo run -p tidyup-cli -- history --format json
```

## Safety Notes

- TidyUp never overwrites an occupied destination path.
- TidyUp revalidates actions immediately before execution.
- Undo is also validation-driven and may refuse unsafe restoration.
- Plans and rule packs are treated as untrusted inputs.
- TidyUp warns when you run it in a project-like folder because repositories are usually a poor fit for the tool.

See the docs in [docs/](/Users/niketkakkar/.codex/worktrees/a633/tidyup/docs) for the architecture, safety model, plan format, journal format, CLI contract, and rule-pack details.

## Installation And Releases

Release artifacts are built for macOS and Windows by the release workflow in [.github/workflows/release.yml](/Users/niketkakkar/.codex/worktrees/a633/tidyup/.github/workflows/release.yml).

Until a published binary is downloaded, the easiest local path is:

```bash
cargo build --release -p tidyup-cli
./target/release/tidyup scan
```

Windows users can run:

```powershell
.\target\release\tidyup.exe scan
```

## Additional Docs

- [CONTRIBUTING.md](/Users/niketkakkar/.codex/worktrees/a633/tidyup/CONTRIBUTING.md)
- [SECURITY.md](/Users/niketkakkar/.codex/worktrees/a633/tidyup/SECURITY.md)
- [GOVERNANCE.md](/Users/niketkakkar/.codex/worktrees/a633/tidyup/GOVERNANCE.md)
- [CODE_OF_CONDUCT.md](/Users/niketkakkar/.codex/worktrees/a633/tidyup/CODE_OF_CONDUCT.md)
- [ROADMAP.md](/Users/niketkakkar/.codex/worktrees/a633/tidyup/ROADMAP.md)
- [RELEASE_CHECKLIST.md](/Users/niketkakkar/.codex/worktrees/a633/tidyup/RELEASE_CHECKLIST.md)
- [CHANGELOG.md](/Users/niketkakkar/.codex/worktrees/a633/tidyup/CHANGELOG.md)
- [docs/INSTALLATION.md](/Users/niketkakkar/.codex/worktrees/a633/tidyup/docs/INSTALLATION.md)
