# TidyUp

TidyUp is a safe file-organization command-line tool.

It helps you clean up a folder by:

- showing what it would move first
- asking before it changes anything
- recording what happened
- letting you undo a safe operation later

The simplest way to think about it is:

```text
"organize this folder carefully, and let me undo it if needed"
```

The MVP promise is narrow on purpose:

- organize one selected folder safely
- preview before mutation
- never overwrite existing files silently
- record what happened
- undo safely when restoration is still valid

TidyUp is not a generic cleanup script and it is not meant to reorganize an entire code repository automatically.

For developers, the safest use is:

- exported folders
- download folders
- screenshots folders
- data drop folders
- asset intake folders

It is usually not a good idea to point TidyUp at the root of your codebase unless you intentionally want that folder reorganized.

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

Current presentation status:

- macOS is the active MVP path
- Windows packaging and release validation are deferred to later issues

## What Kind Of Software Is This?

TidyUp is a CLI utility, not a framework.

You run commands in a terminal like:

```bash
tidyup scan
tidyup plan
tidyup apply
```

So if you want to use it with your own codebase later, the natural model is:

- install the tool
- run it from the terminal
- or call it from scripts/automation
- or use its JSON output from another program

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

If the preview looks right, confirm the prompt.

If you are scripting it, use `--yes` for non-interactive runs.

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

## Using TidyUp In A Codebase

If you are a developer, the safest pattern is to treat TidyUp like an external tool.

Good examples:

- clean up a generated reports folder
- organize downloaded fixtures before importing them
- sort screenshots or exported files
- run `plan --format json` inside a script to inspect what would happen

Less safe examples:

- running it at the repo root
- using it on folders where file layout is part of the build system
- using it on folders with hand-curated source trees

Recommended automation pattern:

```bash
tidyup plan --root ./incoming-assets --format json
tidyup apply --root ./incoming-assets --yes
```

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

The current MVP should be presented as macOS-first.

The repository still contains Windows-oriented workflow scaffolding, but Windows distribution should be treated as future work until it is validated and intentionally released.

Until a published binary is downloaded, the easiest local path is:

```bash
cargo build --release -p tidyup-cli
./target/release/tidyup scan
```

For the current MVP demo and showcase, use the macOS binary path above.

## In One Sentence

TidyUp is a cautious folder-cleanup tool for people who want preview, audit history, and undo instead of risky one-shot file moves.

## Additional Docs

- [CONTRIBUTING.md](/Users/niketkakkar/.codex/worktrees/a633/tidyup/CONTRIBUTING.md)
- [SECURITY.md](/Users/niketkakkar/.codex/worktrees/a633/tidyup/SECURITY.md)
- [GOVERNANCE.md](/Users/niketkakkar/.codex/worktrees/a633/tidyup/GOVERNANCE.md)
- [CODE_OF_CONDUCT.md](/Users/niketkakkar/.codex/worktrees/a633/tidyup/CODE_OF_CONDUCT.md)
- [ROADMAP.md](/Users/niketkakkar/.codex/worktrees/a633/tidyup/ROADMAP.md)
- [RELEASE_CHECKLIST.md](/Users/niketkakkar/.codex/worktrees/a633/tidyup/RELEASE_CHECKLIST.md)
- [CHANGELOG.md](/Users/niketkakkar/.codex/worktrees/a633/tidyup/CHANGELOG.md)
- [docs/INSTALLATION.md](/Users/niketkakkar/.codex/worktrees/a633/tidyup/docs/INSTALLATION.md)
- [docs/RELEASE_EVIDENCE_v0.1.0.md](/Users/niketkakkar/.codex/worktrees/a633/tidyup/docs/RELEASE_EVIDENCE_v0.1.0.md)
