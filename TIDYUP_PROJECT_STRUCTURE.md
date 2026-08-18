# TidyUp — Master Project Structure and Engineering Specification

> **Status:** Initial source-of-truth specification  
> **Working project name:** `tidyup`  
> **Primary implementation language:** Rust  
> **Initial supported platforms:** Windows and macOS  
> **Initial interface:** Guided CLI/TUI  
> **Project type:** Local-first, deterministic, reversible filesystem organization engine  
> **Initial release target:** `v0.1.0`

---

## 1. Project Thesis

TidyUp is a local-first file-organization system designed around one principle:

> **No filesystem change should occur unless the user can understand it beforehand, the application can validate it immediately before execution, and the completed operation can be safely audited and reversed.**

The project is not intended to be a generic “cleanup script.” Its core value is a reusable filesystem planning and execution engine that separates scanning, classification, planning, validation, approval, execution, journaling, and undo into explicit stages.

The organizer CLI is the first application of that engine.

The long-term open-source value is therefore broader than sorting files. TidyUp should provide:

- A reusable filesystem-planning core.
- A deterministic rule engine.
- Inspectable and versioned plan formats.
- Guarded filesystem execution.
- Append-only operation history.
- Conflict-safe undo.
- Cross-platform filesystem abstractions.
- Stable structured output for automation.
- Declarative community rule packs.
- A filesystem test toolkit for contributors.
- A safe foundation for future CLI, TUI, desktop, or third-party integrations.

---

## 2. Problem Statement

Personal folders such as Downloads, Desktop, project drop folders, exported-data folders, and shared working directories often accumulate large numbers of files.

Existing cleanup approaches commonly fail users in one of several ways:

1. They require command-line or scripting knowledge.
2. They make destructive changes too quickly.
3. They hide why a file was moved.
4. They overwrite or rename files automatically.
5. They behave differently across operating systems.
6. They provide weak or nonexistent recovery.
7. They depend on cloud processing or upload metadata.
8. They combine organization, deletion, duplicate cleanup, sync, and backup into an unpredictable workflow.
9. They are difficult for outside contributors to extend safely.

TidyUp addresses this by treating file organization as a **planned, reviewable, validated, journaled filesystem operation** rather than a one-shot script.

---

## 3. Primary Users

### 3.1 Novice end user

A person who wants to organize a folder without learning file-management commands.

They should be able to:

- Install TidyUp.
- Select a folder.
- Understand that scanning is read-only.
- See category summaries.
- Preview proposed moves.
- Understand why each move was proposed.
- Approve or reject changes.
- Review completed, skipped, and failed actions.
- Undo a previous operation safely.

### 3.2 Power user

A technically comfortable user who wants:

- Direct CLI commands.
- JSON output.
- Saved plans.
- Script integration.
- Custom declarative rule packs.
- Operation history.
- Rule explanation.
- Explicit non-interactive operation modes.

### 3.3 Open-source contributor

A developer who should be able to contribute to one area without understanding the full codebase.

Contribution surfaces should include:

- Classification rules.
- Rule-pack validation.
- CLI UX.
- Documentation.
- Windows behavior.
- macOS behavior.
- Filesystem fixtures.
- Failure-injection tests.
- Output formatting.
- Packaging.
- Release tooling.
- Security hardening.

### 3.4 Library consumer

A developer who wants to reuse the TidyUp engine without the CLI.

The public core should eventually support:

```text
scan -> classify -> plan -> validate -> execute -> journal -> undo
```

through documented Rust APIs.

---

## 4. Product Principles

### 4.1 Preview before mutation

No modifying operation should be the default path.

### 4.2 Deterministic behavior

The same input state, configuration, and platform semantics should produce the same plan.

### 4.3 Explainability

Every proposed action must identify the rule and reason that produced it.

### 4.4 No silent overwrite

Existing data must never be overwritten merely to satisfy a move or undo.

### 4.5 Revalidation before execution

A plan is a proposal based on a prior filesystem state. It is not authorization to ignore later changes.

### 4.6 Accurate partial-failure reporting

A batch operation must never be represented as fully successful when only some actions completed.

### 4.7 Local-first privacy

TidyUp should not require cloud services for its primary workflow.

### 4.8 Cross-platform correctness is a product feature

Windows and macOS support must influence design from the first implementation, not be treated as a packaging task.

### 4.9 Narrow modification surface

The initial release moves files only. It does not permanently delete, edit file contents, synchronize folders, or reorganize the entire filesystem.

### 4.10 Safe extension

Community extensibility begins with declarative rule packs, not arbitrary executable plugins.

---

## 5. Initial Product Scope

The `v0.1` product should prove one claim:

> A first-time user can safely organize one folder and undo the result without understanding shell commands.

The initial supported loop is:

```text
Choose folder
    ->
Scan
    ->
Classify
    ->
Generate plan
    ->
Validate
    ->
Preview
    ->
Approve
    ->
Revalidate
    ->
Move
    ->
Review result
    ->
Undo
```

---

## 6. Explicit v0.1 Boundaries

### 6.1 Allowed

- One user-selected root directory.
- Direct child files of that root.
- Deterministic classification by file metadata, initially extensions.
- Destination subfolders inside the selected root.
- Read-only scanning.
- Plan generation.
- Plan preview.
- Per-action approval.
- Same-scope file moves.
- Operation journaling.
- Guarded undo.
- Human-readable CLI output.
- Stable JSON output for read-only commands where specified.
- Windows support.
- macOS support.

### 6.2 Not allowed in v0.1

- Permanent deletion.
- Automatic trash/recycle-bin cleanup.
- Recursive reorganization of arbitrary directory trees.
- Moving files to unrelated locations outside the selected root.
- Cloud processing.
- AI classification.
- File-content analysis unless later explicitly justified.
- Executable plugins.
- Shell hooks.
- Scheduled cleanup.
- Background agents.
- Duplicate deletion.
- File synchronization.
- Backup.
- Automatic collision renaming.
- Cross-volume copy-and-delete behavior.
- Automatic following of symbolic links, aliases, junctions, or reparse points.
- Arbitrary user scripts inside rule packs.
- GUI/desktop application.

---

## 7. v0.1 Observable Behavior

A complete acceptance scenario should look like this:

1. User launches `tidyup`.
2. User selects or enters a folder.
3. TidyUp explains that scanning does not change files.
4. TidyUp scans supported direct-child entries.
5. Unsupported or risky entries are skipped with explanations.
6. Each supported file is classified by deterministic built-in rules.
7. TidyUp proposes destination subfolders.
8. TidyUp checks for collisions and invalid destinations.
9. User reviews a summary.
10. User reviews individual proposed moves.
11. User approves selected actions.
12. Immediately before each move, TidyUp revalidates source and destination state.
13. Valid approved files are moved.
14. Skipped and failed actions are recorded independently.
15. TidyUp shows the exact result.
16. The operation is written to history.
17. User can request undo.
18. Undo revalidates current filesystem state.
19. Safe actions are restored.
20. Unsafe undo actions stop without overwriting data.

---

## 8. Core Architecture

TidyUp should use explicit architectural boundaries.

```text
                         +------------------+
                         |    CLI / TUI     |
                         +--------+---------+
                                  |
                         +--------v---------+
                         | Application Flow |
                         +--------+---------+
                                  |
              +-------------------+-------------------+
              |                   |                   |
       +------v------+      +-----v------+      +-----v------+
       |   Scanner   | ---> | Rule Engine | ---> |  Planner   |
       +-------------+      +-------------+      +-----+------+
                                                        |
                                                 +------v------+
                                                 |  Validator  |
                                                 +------+------+
                                                        |
                                                 +------v------+
                                                 |  Executor   |
                                                 +------+------+
                                                        |
                        +-------------------------------+----------------+
                        |                               |                |
                 +------v------+                 +------v------+   +-----v-----+
                 |   Journal   |                 |    Undo     |   |  Events   |
                 +-------------+                 +-------------+   +-----------+

                                  |
                         +--------v---------+
                         | Platform Adapter |
                         | Windows / macOS  |
                         +------------------+
```

The CLI must not directly implement filesystem mutation logic.

The core must not directly depend on terminal rendering.

The platform layer must isolate OS-specific filesystem behavior where possible.

---

## 9. Proposed Repository Structure

```text
tidyup/
|
|-- Cargo.toml
|-- Cargo.lock
|
|-- crates/
|   |
|   |-- tidyup-core/
|   |   |-- Cargo.toml
|   |   `-- src/
|   |       |-- lib.rs
|   |       |
|   |       |-- domain/
|   |       |   |-- mod.rs
|   |       |   |-- file_snapshot.rs
|   |       |   |-- classification.rs
|   |       |   |-- rule.rs
|   |       |   |-- plan.rs
|   |       |   |-- operation.rs
|   |       |   `-- identifiers.rs
|   |       |
|   |       |-- scanner/
|   |       |   |-- mod.rs
|   |       |   `-- scanner.rs
|   |       |
|   |       |-- rules/
|   |       |   |-- mod.rs
|   |       |   |-- builtin.rs
|   |       |   |-- matcher.rs
|   |       |   |-- parser.rs
|   |       |   `-- validation.rs
|   |       |
|   |       |-- planner/
|   |       |   |-- mod.rs
|   |       |   |-- planner.rs
|   |       |   `-- conflict.rs
|   |       |
|   |       |-- validator/
|   |       |   |-- mod.rs
|   |       |   |-- plan_validator.rs
|   |       |   |-- action_validator.rs
|   |       |   `-- stale.rs
|   |       |
|   |       |-- executor/
|   |       |   |-- mod.rs
|   |       |   |-- executor.rs
|   |       |   `-- result.rs
|   |       |
|   |       |-- undo/
|   |       |   |-- mod.rs
|   |       |   |-- planner.rs
|   |       |   `-- validator.rs
|   |       |
|   |       `-- error.rs
|   |
|   |-- tidyup-platform/
|   |   |-- Cargo.toml
|   |   `-- src/
|   |       |-- lib.rs
|   |       |-- path.rs
|   |       |-- metadata.rs
|   |       |-- filesystem.rs
|   |       |-- app_dirs.rs
|   |       |-- windows.rs
|   |       `-- macos.rs
|   |
|   |-- tidyup-storage/
|   |   |-- Cargo.toml
|   |   `-- src/
|   |       |-- lib.rs
|   |       |-- database.rs
|   |       |-- journal.rs
|   |       |-- repository.rs
|   |       `-- migrations/
|   |
|   |-- tidyup-cli/
|   |   |-- Cargo.toml
|   |   `-- src/
|   |       |-- main.rs
|   |       |-- app.rs
|   |       |
|   |       |-- commands/
|   |       |   |-- mod.rs
|   |       |   |-- scan.rs
|   |       |   |-- plan.rs
|   |       |   |-- apply.rs
|   |       |   |-- history.rs
|   |       |   |-- undo.rs
|   |       |   `-- rules.rs
|   |       |
|   |       |-- interactive/
|   |       |   |-- mod.rs
|   |       |   |-- folder.rs
|   |       |   |-- preview.rs
|   |       |   |-- approval.rs
|   |       |   `-- summary.rs
|   |       |
|   |       `-- output/
|   |           |-- mod.rs
|   |           |-- human.rs
|   |           |-- json.rs
|   |           `-- errors.rs
|   |
|   `-- tidyup-testkit/
|       |-- Cargo.toml
|       `-- src/
|           |-- lib.rs
|           |-- fixture.rs
|           |-- scenario.rs
|           |-- failure.rs
|           `-- assertions.rs
|
|-- rule-packs/
|   `-- default/
|       |-- rules.toml
|       |-- README.md
|       `-- fixtures/
|
|-- tests/
|   |-- acceptance/
|   |-- filesystem/
|   |-- failure-injection/
|   |-- golden/
|   `-- fixtures/
|
|-- examples/
|   |-- plans/
|   |-- rules/
|   `-- scripts/
|
|-- docs/
|   |-- product.md
|   |-- architecture.md
|   |-- safety-model.md
|   |-- threat-model.md
|   |-- filesystem-semantics.md
|   |-- plan-format.md
|   |-- journal-format.md
|   |-- rule-pack-spec.md
|   |-- platform-support.md
|   |-- cli-contract.md
|   |-- testing-strategy.md
|   |-- release-process.md
|   `-- adr/
|       |-- 0001-rust-workspace.md
|       |-- 0002-no-overwrite-policy.md
|       |-- 0003-selected-root-containment.md
|       |-- 0004-sqlite-operation-journal.md
|       `-- 0005-declarative-rules-only.md
|
|-- scripts/
|
|-- .github/
|   |-- workflows/
|   |   |-- ci.yml
|   |   `-- release.yml
|   |-- ISSUE_TEMPLATE/
|   |   |-- bug.yml
|   |   |-- feature.yml
|   |   `-- platform_bug.yml
|   |-- PULL_REQUEST_TEMPLATE.md
|   `-- dependabot.yml
|
|-- README.md
|-- CONTRIBUTING.md
|-- SECURITY.md
|-- GOVERNANCE.md
|-- CODE_OF_CONDUCT.md
|-- ROADMAP.md
|-- CHANGELOG.md
|-- LICENSE
|-- AGENTS.md
|-- rustfmt.toml
|-- clippy.toml
|-- deny.toml
`-- .gitignore
```

---

## 10. Crate Responsibilities

### 10.1 `tidyup-core`

Owns platform-neutral business and safety logic.

Responsibilities:

- Domain models.
- Scan result representation.
- Classification.
- Rule evaluation.
- Plan generation.
- Conflict modelling.
- Plan validation logic.
- Action validation contracts.
- Execution orchestration contracts.
- Undo planning.
- Core error taxonomy.

Must not own:

- Terminal prompts.
- Terminal colors.
- OS-specific UI.
- SQLite details.
- OS-specific configuration locations.

### 10.2 `tidyup-platform`

Owns OS-sensitive behavior.

Responsibilities:

- Path comparison semantics.
- File identity extraction.
- Platform-aware metadata.
- Windows reserved-name checks.
- macOS-specific path and link semantics.
- Link/reparse/junction detection.
- Filesystem mutation primitives.
- User configuration/history directory discovery.
- Long-path handling.
- Same-volume detection.
- Platform feature capability reporting.

### 10.3 `tidyup-storage`

Owns persistent local state.

Responsibilities:

- Operation journal.
- Database migrations.
- Operation queries.
- Action-result persistence.
- Schema versioning.
- History retrieval.
- Durable undo references.

### 10.4 `tidyup-cli`

Owns user interaction.

Responsibilities:

- Command parsing.
- Guided mode.
- Folder selection/input.
- Read-only summaries.
- Plan preview.
- Confirmation.
- Structured output.
- Exit codes.
- Recovery instructions.

Must call the same core APIs used by any future GUI.

### 10.5 `tidyup-testkit`

Owns reusable filesystem test helpers.

Responsibilities:

- Temporary fixture creation.
- Unicode filename fixtures.
- Collision scenarios.
- Permission scenarios where platform-supported.
- Change-after-plan scenarios.
- Failure injection.
- Operation assertions.
- Platform-normalized test expectations.

---

## 11. Domain Model

The project should use typed domain objects rather than loose maps.

### 11.1 `FileSnapshot`

Represents filesystem state observed during scan or validation.

Conceptual fields:

```text
FileSnapshot
|-- path
|-- normalized_comparison_key
|-- file_identity
|-- size
|-- modified_time
|-- entry_type
|-- link_state
|-- platform_metadata
`-- warnings
```

`file_identity` should use the strongest stable identifier that can be obtained safely on the active platform.

A metadata fingerprint must not be treated as cryptographic proof. It is a stale-plan guard.

### 11.2 `Classification`

```text
Classification
|-- category_id
|-- category_label
|-- rule_id
|-- reason
`-- confidence = deterministic
```

There should be no probabilistic classification in v0.1.

### 11.3 `PlannedMove`

```text
PlannedMove
|-- action_id
|-- source
|-- source_snapshot
|-- destination
|-- matched_rule_id
|-- matched_rule_version
|-- reason
|-- warnings
|-- validation_status
`-- approval_status
```

### 11.4 `Plan`

```text
Plan
|-- schema_version
|-- plan_id
|-- application_version
|-- root
|-- created_at
|-- platform
|-- ruleset_version
|-- scan_summary
`-- actions[]
```

A plan must be immutable after it is persisted or exported.

Changes should result in a new plan.

### 11.5 `ActionResult`

```text
ActionResult
|-- action_id
|-- source
|-- destination
|-- outcome
|-- reason_code
|-- started_at
|-- finished_at
|-- error_details
`-- post_action_snapshot
```

Valid outcome states:

```text
completed
skipped
failed
cancelled
```

### 11.6 `Operation`

```text
Operation
|-- operation_id
|-- operation_type
|-- plan_id
|-- parent_operation_id
|-- started_at
|-- finished_at
|-- platform
|-- application_version
|-- root
`-- results[]
```

`operation_type` initially supports:

```text
apply
undo
```

Undo is recorded as a new operation. History is not rewritten.

---

## 12. Scanner Design

The scanner is read-only.

### Responsibilities

- Accept a selected root.
- Validate that the root exists and is accessible.
- Inspect direct-child entries.
- Identify supported regular files.
- Identify unsupported or risky entries.
- Record metadata snapshots.
- Return warnings and skipped entries.
- Never mutate filesystem state.

### v0.1 entry policy

| Entry | Behavior |
|---|---|
| Regular file | Scan |
| Directory | Ignore for reorganization; may report |
| Symbolic link | Skip and warn |
| Windows junction/reparse point | Skip and warn |
| macOS alias | Skip and warn |
| Special file | Skip and warn |
| Inaccessible entry | Skip and report |
| Hidden file | Scan only according to explicit product policy; default should be conservative |
| OS metadata file | Skip when known and documented |

The scanner should avoid reading file contents when metadata is sufficient.

---

## 13. Classification and Rule Engine

### 13.1 v0.1 classification

Initial classification is extension-based.

Example categories:

```text
Documents
Images
Video
Audio
Archives
Code
Data
Installers
Other / Unclassified
```

Exact category names should be treated as product-level configuration rather than hardcoded across the codebase.

### 13.2 Rule principles

Rules must be:

- Deterministic.
- Declarative.
- Inspectable.
- Versioned.
- Validated before activation.
- Contained to the selected root.
- Unable to execute arbitrary code.

### 13.3 Example rule pack

```toml
schema_version = 1
pack_id = "default"
pack_version = "0.1.0"

[[rules]]
id = "documents.pdf"
description = "Organize PDF documents"
extensions = ["pdf"]
destination = "Documents"
priority = 100

[[rules]]
id = "images.common"
description = "Organize common image formats"
extensions = ["jpg", "jpeg", "png", "gif", "webp"]
destination = "Images"
priority = 100
```

### 13.4 Rule validation requirements

Validate:

- Schema version.
- Required fields.
- Rule ID uniqueness.
- Valid destination.
- Destination containment.
- Invalid path characters.
- Reserved Windows names where relevant.
- Duplicate rules.
- Conflicting priorities.
- Unsupported fields.
- Unsupported actions.
- Pack-version compatibility.

### 13.5 Rule precedence

Initial policy:

1. Higher numeric priority wins.
2. Equal-priority conflicting matches produce a conflict.
3. Conflict produces no executable move.
4. Rule order in the file must not silently resolve semantic conflicts.

### 13.6 Rule explanation

Future command:

```shell
tidyup rules explain path-to-file
```

Example output:

```text
Matched rule:
  documents.pdf

Reason:
  Extension ".pdf" matched rule extensions.

Proposed destination:
  Documents/report.pdf

Rules evaluated:
  12

Other matching rules:
  none
```

---

## 14. Planner Design

The planner converts scan results into proposed actions without modifying the filesystem.

### Planner responsibilities

- Apply rules to supported files.
- Propose destination subfolders.
- Preserve the original filename.
- Keep destinations inside the selected root.
- Detect same-plan target collisions.
- Detect existing destination collisions.
- Detect invalid names.
- Mark warnings.
- Produce a serializable plan.
- Never execute a move.

### Planner invariants

A planned destination must:

- Be inside the selected root.
- Not contain traversal outside scope.
- Not target the source path.
- Be valid for the active platform.
- Not rely on case-only distinction on a case-insensitive filesystem.
- Not overwrite an existing path.
- Not target the same path as another plan action.

---

## 15. Validation Model

Validation occurs in more than one stage.

### 15.1 Plan-time validation

Performed after plan generation.

Checks:

- Root containment.
- Destination syntax.
- Platform validity.
- Existing collisions.
- Internal plan collisions.
- Unsupported source type.
- Rule validity.

### 15.2 Execution-time revalidation

Performed immediately before each action.

Checks:

- Source still exists.
- Source is still the same relevant filesystem object.
- Source metadata has not materially changed.
- Source has not become a link/special entry.
- Destination still does not exist.
- Destination parent is still valid.
- Destination remains inside selected scope.
- Current platform constraints still pass.

A plan that passed validation earlier can still be rejected at execution time.

### 15.3 Suggested machine-readable reason codes

```text
STALE_SOURCE
SOURCE_MISSING
SOURCE_TYPE_CHANGED
DESTINATION_EXISTS
DESTINATION_INVALID
DESTINATION_OUTSIDE_SCOPE
PLAN_TARGET_COLLISION
RULE_CONFLICT
PERMISSION_DENIED
FILE_LOCKED
UNSUPPORTED_ENTRY
UNSUPPORTED_CROSS_VOLUME_MOVE
USER_REJECTED
USER_CANCELLED
PLATFORM_ERROR
IO_ERROR
```

Reason codes should remain stable once exposed through JSON output.

---

## 16. Stale-Plan Protection

Stale-plan detection is a first-class safety feature.

Example:

1. TidyUp scans `report.pdf`.
2. It records a `FileSnapshot`.
3. A plan proposes moving `report.pdf`.
4. Before apply, the user replaces the file.
5. The source no longer matches the planned snapshot.
6. Execution rejects the action with `STALE_SOURCE`.

A source fingerprint may include:

- Platform filesystem identity where available.
- Size.
- Modification timestamp.
- Entry type.
- Additional safe metadata when useful.

Destination existence must always be rechecked immediately before the move.

No validation performed during planning may be assumed to remain valid during execution.

---

## 17. Executor Design

The executor applies approved actions one at a time.

### v0.1 constraints

- Move files only.
- Do not delete.
- Do not overwrite.
- Do not edit contents.
- Do not follow links.
- Do not reorganize directories.
- Do not automatically rename collisions.
- Reject unsupported cross-volume moves.

### Per-action execution loop

```text
Approved action
    ->
Revalidate source
    ->
Revalidate destination
    ->
Perform move
    ->
Capture result
    ->
Persist result
    ->
Continue if safe
```

### Failure policy

A failure must not automatically trigger risky rollback.

Instead:

- Record the completed actions.
- Record the skipped actions.
- Record the failed action.
- Continue only when remaining actions remain independently safe.
- Present accurate partial results.

---

## 18. Operation Journal

The journal is part of the safety architecture.

SQLite is the recommended initial persistence mechanism because it provides:

- Transactions.
- Schema migration.
- Structured history.
- Queryability.
- Reliable single-user local persistence.
- Cross-platform availability.

### 18.1 Conceptual schema

```text
operations
|-- operation_id
|-- operation_type
|-- plan_id
|-- parent_operation_id
|-- root
|-- platform
|-- app_version
|-- started_at
`-- finished_at

action_results
|-- result_id
|-- operation_id
|-- action_id
|-- source_path
|-- destination_path
|-- outcome
|-- reason_code
|-- error_text
|-- started_at
|-- finished_at
`-- metadata_json
```

### 18.2 Journal requirements

- Append operation history rather than rewriting it.
- Preserve partial operations.
- Persist actual results, not only intended actions.
- Support history retrieval.
- Support undo planning.
- Use migrations.
- Document storage location.
- Avoid storing unnecessary file-content information.

---

## 19. Undo Design

Undo reverses only actions that actually completed.

### Example

Apply operation:

```text
10 approved
7 completed
1 stale
1 collision
1 permission failure
```

Undo targets only the seven completed actions.

### Undo validation

Before restoring each file:

- The moved file must still exist at its current location.
- It must still correspond to the operation being undone.
- The original path must be safe.
- The original path must not contain another file.
- The destination file must not have materially changed in a way that makes restoration unsafe.

### Undo invariant

> Undo must never overwrite a newer or unrelated file merely to restore an older location.

### Conflict example

```text
Original apply:
Downloads/report.pdf
    ->
Downloads/Documents/report.pdf

Later:
A new Downloads/report.pdf appears.

Undo:
STOP that action.
Report conflict.
Do not overwrite the new Downloads/report.pdf.
```

### Undo journaling

Undo creates a new operation:

```text
operation op_1001: apply
operation op_1002: undo of op_1001
```

Historical records are not deleted.

---

## 20. Cross-Platform Filesystem Requirements

Cross-platform support must be represented in design, code, tests, and CI.

### Windows considerations

- Drive letters.
- UNC/network paths.
- Reserved names.
- Invalid filename characters.
- Case-insensitive/case-preserving behavior.
- Reparse points.
- Junctions.
- File locks.
- Antivirus interference.
- Long paths.
- Volume boundaries.
- Recycle Bin behavior.
- Application data directories.

### macOS considerations

- Case-insensitive/case-preserving filesystems are common.
- Unicode normalization.
- Symbolic links.
- Finder aliases.
- Extended attributes.
- Metadata files.
- Volume boundaries.
- Trash behavior.
- Application support directories.

### Shared invariant

TidyUp must use platform-aware path/filesystem APIs.

It must never build correctness logic from assumptions such as:

```text
path = root + "/" + child
```

or:

```text
Foo.txt != foo.txt
```

without consulting platform semantics.

---

## 21. Path Safety

All externally supplied paths are untrusted input.

Path validation should defend against:

- `..` traversal.
- Absolute destinations from rule packs.
- Root escape after normalization.
- Symlink-assisted escape.
- Junction/reparse-assisted escape.
- Case-equivalent collisions.
- Invalid Windows filenames.
- Unicode ambiguities where relevant.
- Destination parent replacement between planning and execution.

Containment must be checked through platform-aware filesystem/path logic rather than string-prefix comparison.

---

## 22. Privacy and Security

### Default policy

- Process locally.
- Do not upload filenames.
- Do not upload metadata.
- Do not collect telemetry by default.
- Do not follow links outside selected scope.
- Avoid reading file contents when metadata is enough.
- Treat imported plans and rule packs as untrusted input.
- Explain inspected scope.
- Support redacted diagnostics.

### Future analytics

If analytics or crash reporting are introduced:

- They must be opt-in.
- The exact transmitted data must be documented.
- Sensitive paths should be excluded or redacted by default.
- The project must remain fully usable without analytics.

---

## 23. Threat Model

The threat model should cover at least:

### Malicious rule pack

Goal: escape the selected root or trigger unsafe behavior.

Mitigations:

- Declarative schema only.
- Destination containment validation.
- No executable hooks.
- No command interpolation.
- No arbitrary code.

### TOCTOU filesystem changes

Goal: filesystem changes between planning and execution.

Mitigations:

- Action-level revalidation.
- Source fingerprint checks.
- Destination rechecks.
- Refuse stale actions.

### Link redirection

Goal: use symbolic links/junctions to redirect operations.

Mitigations:

- Skip link-like entries in v0.1.
- Validate parent paths.
- Avoid following external links implicitly.

### Collision overwrite

Goal: overwrite an existing file.

Mitigation:

- No-overwrite invariant enforced in planner and executor.

### Unsafe undo

Goal: restore an old file over a new one.

Mitigation:

- Undo revalidation.
- Refuse occupied original paths.
- Refuse unexpected destination changes.

### Interrupted process

Goal: leave inaccurate state after process termination.

Mitigation:

- Persist action results incrementally.
- Journal actual completion.
- Recovery reports partial state rather than pretending batch atomicity.

---

## 24. CLI Design

The guided experience remains the default.

### Phase 1

```shell
tidyup
tidyup path-to-folder
```

### Phase 2 — read-only commands

```shell
tidyup scan path-to-folder
tidyup plan path-to-folder
tidyup history
tidyup history show operation-id
```

Structured output:

```shell
tidyup scan path-to-folder --output json
tidyup plan path-to-folder --output json
```

### Phase 3 — modifying commands

```shell
tidyup apply saved-plan
tidyup undo operation-id
```

Interactive apply requires explicit confirmation.

Non-interactive modification should require an explicit flag and should not be introduced casually.

### Phase 4 — rules

```shell
tidyup rules list
tidyup rules explain path-to-file
tidyup rules validate path-to-rule-pack
```

### Phase 5 — read-only analysis

```shell
tidyup duplicates path-to-folder
tidyup large-files path-to-folder
tidyup old-files path-to-folder
```

These should remain report-first capabilities. Any future modification must flow through the same plan/validate/approve/execute/journal pipeline.

---

## 25. Guided CLI Flow

Example:

```text
TidyUp

Choose a folder:
> ~/Downloads

Scanning makes no changes to your files.

Scan complete:
  36 regular files
  28 classified
   5 unclassified
   3 skipped

Proposed organization:
  Documents   12
  Images       8
  Archives     5
  Video        3

Warnings:
  1 destination collision
  2 unsupported links

Preview all proposed moves? [Y/n]
```

Before execution:

```text
8 moves approved.

TidyUp will move these files inside:
  ~/Downloads

It will not delete or overwrite files.

Apply approved moves? [y/N]
```

After execution:

```text
Operation complete.

7 moved
1 skipped: destination created after preview
0 overwritten

Operation ID:
  op_0193

Undo with:
  tidyup undo op_0193
```

---

## 26. CLI Usability Conventions

- Running without arguments starts guided mode.
- Help contains examples.
- Preview is the default.
- Destructive-sounding commands such as `clean` should be avoided when behavior is only organization.
- Modifying commands clearly state what will change.
- Color is supplementary, not semantically required.
- Output must work in common Windows and macOS terminals.
- Paths with spaces and Unicode must render correctly.
- Machine-readable output must use stable field names.
- Human-readable output may evolve more freely.

---

## 27. Exit Code Contract

Suggested initial contract:

```text
0  success
1  operational failure
2  invalid input/configuration
3  completed with skips or partial failures
4  user cancelled
5  stale plan / validation rejected
```

Exact codes should be documented before release and then treated as compatibility surface.

---

## 28. Structured Output

JSON output should be versioned.

Example:

```json
{
  "schema_version": 1,
  "command": "scan",
  "root": "/example/Downloads",
  "summary": {
    "files": 12,
    "classified": 10,
    "unclassified": 1,
    "skipped": 1
  }
}
```

Breaking JSON schema changes require explicit schema-version handling.

---

## 29. Testing Strategy

Testing is part of the product, especially because the software changes user files.

### 29.1 Unit tests

Cover:

- Rule parsing.
- Rule matching.
- Priority resolution.
- Path validation.
- Reason codes.
- Domain invariants.
- Plan generation.

### 29.2 Property-based tests

Useful targets:

- Planner never emits path outside root.
- Generated destination cannot equal source unexpectedly.
- Conflicting plans cannot silently overwrite.
- Rule ordering does not change deterministic conflict behavior.

### 29.3 Filesystem integration tests

Use temporary disposable directories.

Cover:

- Normal moves.
- Spaces.
- Unicode names.
- Hidden entries.
- Existing destination.
- Source missing.
- Destination appearing after plan.
- Source modification after plan.
- Permission failure.
- File lock where reproducible.
- Unsupported link.
- Long path.
- Deep path.
- Case collision.

### 29.4 Cross-platform acceptance tests

Run on current supported Windows and macOS GitHub Actions runners.

Equivalent logical fixtures should produce equivalent semantic outcomes, excluding explicitly documented OS metadata differences.

### 29.5 Failure-injection tests

Inject failures:

```text
before first move
after validation
during move
after filesystem move but before result persistence
between action 3 and action 4
during undo
```

The goal is to ensure that the journal and user-visible result remain truthful.

### 29.6 Packaged-binary smoke tests

Test release artifacts on clean environments without Rust tooling.

Verify:

- Program starts.
- Help works.
- Scan works.
- Plan works.
- Apply works on disposable data.
- Undo works.
- Configuration/history directories are correct.

---

## 30. High-Value Safety Test Matrix

Required scenarios should include:

```text
plan -> source changes -> apply
plan -> source replaced -> apply
plan -> destination appears -> apply
plan -> second action targets same destination
move -> original path becomes occupied -> undo
move -> destination changes -> undo
move -> process interrupted midway
Unicode composed/decomposed filenames
Foo.txt vs foo.txt
Windows reserved filenames
very long path
spaces in every path component
permission revoked after planning
unsupported symbolic link
unsupported Windows junction
unsupported macOS alias
cross-volume move requested
locked file
removable-volume disappearance
```

A safety claim is not complete until represented by automated evidence where technically feasible.

---

## 31. CI Structure

Initial CI should run:

### Every pull request

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- Unit tests.
- Integration tests.
- Windows acceptance tests.
- macOS acceptance tests.
- Documentation build.
- Dependency/security policy checks.

### Optional Linux CI

Linux may be useful for fast developer feedback and core portability, but Windows and macOS remain required product targets.

### Main-branch verification

- Full test suite.
- Golden output tests.
- Migration tests.
- Rule-pack validation.
- Packaging dry run where practical.

---

## 32. Release Engineering

Long-term release targets:

### Windows

- Portable executable.
- Installer.
- Signing when resources permit.
- Checksums.
- Documented uninstall and data locations.

### macOS

- Portable binary or application package as appropriate.
- Code signing.
- Notarization when resources permit.
- Checksums.
- Documented uninstall and data locations.

### Advanced users

Potential package-manager distribution:

- Homebrew.
- WinGet or Scoop.
- Cargo installation where appropriate.

Release artifacts should include:

- Version.
- Platform/architecture.
- Checksums.
- Changelog.
- Compatibility notes.
- Known limitations.

---

## 33. Versioning Policy

Use Semantic Versioning.

Before `1.0`:

- Public schemas still require explicit versioning.
- Safety behavior changes must be documented.
- Rule-pack compatibility must be tracked.
- Plan and journal migrations must be deliberate.

Version independently where useful:

```text
application version
plan schema version
journal schema version
rule-pack schema version
```

They should not be assumed to move in lockstep.

---

## 34. Documentation Structure

### `README.md`

Purpose:

- What TidyUp is.
- Why it exists.
- Safety promise.
- 60-second demo.
- Installation.
- Basic workflow.
- Links to deeper docs.

### `docs/product.md`

- Personas.
- Use cases.
- Scope.
- Non-goals.
- Success metrics.

### `docs/architecture.md`

- Components.
- Boundaries.
- Dependency direction.
- Data flow.

### `docs/safety-model.md`

- Invariants.
- Planning.
- Revalidation.
- Execution.
- Failure semantics.
- Undo.

### `docs/threat-model.md`

- Path attacks.
- Rule-pack abuse.
- TOCTOU.
- Links.
- Crash consistency.
- Diagnostics/privacy.

### `docs/filesystem-semantics.md`

- Windows path semantics.
- macOS path semantics.
- Case handling.
- Unicode.
- links.
- locks.
- long paths.
- volumes.

### `docs/rule-pack-spec.md`

- Schema.
- Matching.
- Precedence.
- Validation.
- Compatibility.

### `docs/plan-format.md`

- Plan fields.
- Schema version.
- Persistence/export.
- Compatibility.

### `docs/journal-format.md`

- Database structure.
- Migrations.
- Operation states.
- Undo linkage.

### `docs/testing-strategy.md`

- Test layers.
- Fixtures.
- Platform matrix.
- Failure injection.

### `ROADMAP.md`

- Milestones.
- Evidence gates.
- Out-of-scope items.

---

## 35. Architecture Decision Records

Initial ADRs should be written before or during implementation.

### ADR-0001 — Rust workspace

Decision:
Use Rust for the core and initial CLI.

Rationale:
Cross-platform binaries, explicit error handling, robust library ecosystem, filesystem suitability, strong compile-time guarantees, and reusable core architecture.

### ADR-0002 — No overwrite

Decision:
TidyUp never overwrites an existing destination during apply or undo.

### ADR-0003 — Selected-root containment

Decision:
v0.1 move destinations must remain inside the user-selected root.

### ADR-0004 — SQLite journal

Decision:
Persist operation history in local SQLite storage.

### ADR-0005 — Declarative rule packs

Decision:
Community extensions use validated declarative configuration rather than arbitrary executable plugin code.

---

## 36. Open-Source Repository Files

Required root documentation:

```text
README.md
CONTRIBUTING.md
SECURITY.md
GOVERNANCE.md
CODE_OF_CONDUCT.md
ROADMAP.md
CHANGELOG.md
LICENSE
AGENTS.md
```

### License

Recommended initial choice:

**Apache-2.0**

Reason:
A permissive license with an explicit patent grant is appropriate for a reusable developer-facing engine.

MIT remains a valid simpler alternative if preferred by the maintainer.

---

## 37. Contribution Model

Contributing should be possible without knowing the whole repository.

Examples of self-contained contribution categories:

```text
area:cli
area:rules
area:windows
area:macos
area:storage
area:safety
area:docs
area:testing
area:packaging
good first issue
help wanted
security
```

Safety-sensitive changes should require stronger review.

Examples:

- Overwrite behavior.
- Path containment.
- Link handling.
- Undo.
- Journal persistence.
- Rule destination semantics.
- Filesystem mutation primitives.

---

## 38. `AGENTS.md` Development Discipline

AI-assisted changes should follow a strict repository workflow.

Suggested rules:

1. Inspect relevant code before editing.
2. Understand the safety invariant affected by the change.
3. Reproduce bugs before patching.
4. Keep competing hypotheses during debugging until evidence resolves them.
5. Never weaken overwrite or path-containment protections merely to satisfy tests.
6. Add regression tests for filesystem defects.
7. Use disposable fixtures rather than personal folders.
8. Run platform-relevant tests before claiming completion.
9. Treat plans and rule packs as untrusted input.
10. Record unresolved findings explicitly.
11. Require evidence before marking safety work complete.
12. Verify rendered CLI output or executable behavior where relevant.

For multi-step implementation work:

```text
goal
    ->
implementation
    ->
evidence
    ->
open findings
    ->
verification gate
    ->
completion
```

---

## 39. Milestone Roadmap

### M0 — Specification and Test Infrastructure

Deliver:

- Domain definitions.
- Safety invariants.
- Repository structure.
- Architecture docs.
- Testkit skeleton.
- Cross-platform fixture definitions.
- ADRs.

Completion gate:

- Core observable behavior is represented by acceptance tests or executable test specifications before real file mutation exists.

### M1 — Read-Only Engine

Deliver:

- Scanner.
- Built-in rules.
- Classification.
- Planner.
- Plan validation.
- Human-readable preview.
- JSON scan/plan output.

No filesystem mutation.

Completion gate:

- Equivalent fixtures produce equivalent semantic plans on Windows and macOS.

### M2 — Safe Execution

Deliver:

- Approval handling.
- Execution-time revalidation.
- Same-root moves.
- Per-action results.
- SQLite journal.
- Partial-failure reporting.

Completion gate:

- Stale sources and destination collisions never overwrite data.
- Interrupted/partial operations preserve accurate records.

### M3 — Guarded Undo

Deliver:

- Undo planner.
- Undo validation.
- Undo execution.
- Undo journaling.
- Conflict reporting.

Completion gate:

- Normal and adversarial undo scenarios pass on Windows and macOS.
- Undo never overwrites a conflicting file.

### M4 — Complete User-Facing CLI

Deliver:

```shell
tidyup
tidyup scan
tidyup plan
tidyup apply
tidyup history
tidyup history show
tidyup undo
```

Completion gate:

- First-time tester can organize and reverse a disposable folder using only on-screen instructions.

### M5 — Open-Source `v0.1.0`

Deliver:

- README.
- Contribution guide.
- Security policy.
- Governance.
- Rule documentation.
- Installable/portable builds.
- Checksums.
- Changelog.
- Release notes.
- Beginner issues.

Completion gate:

- An outside developer can clone the repository, run tests, understand architecture, make a small contribution, and submit a PR without private guidance.

### M6 — Stable Rule Packs

Deliver:

```shell
tidyup rules list
tidyup rules explain
tidyup rules validate
```

Completion gate:

- Third-party rule packs can be inspected and validated without executable code.

### M7 — Additional Read-Only Analysis

Potential features:

```shell
tidyup duplicates
tidyup large-files
tidyup old-files
```

These should be added one at a time.

---

## 40. Initial GitHub Issue Backlog

Suggested first issues after repository initialization:

### Foundation

1. Initialize Rust workspace and crate boundaries.
2. Add formatting, linting, and baseline CI.
3. Document core safety invariants.
4. Add ADR-0001 through ADR-0005.
5. Build disposable filesystem fixture testkit.

### Scanner

6. Define `FileSnapshot`.
7. Implement read-only direct-child scanner.
8. Detect unsupported link-like entries.
9. Normalize scanner warnings.
10. Add Windows scanner fixtures.
11. Add macOS scanner fixtures.

### Rules

12. Define rule-pack schema v1.
13. Implement built-in extension rules.
14. Implement rule validation.
15. Implement deterministic priority/conflict behavior.
16. Add rule explanation data model.

### Planner

17. Define `Plan` and `PlannedMove`.
18. Implement selected-root containment.
19. Detect existing destination collisions.
20. Detect same-plan collisions.
21. Detect invalid destination names.
22. Serialize plan schema v1.

### Validation

23. Implement source snapshot comparison.
24. Implement execution-time destination recheck.
25. Define stable validation reason codes.
26. Add stale-plan regression fixtures.

### Executor

27. Implement safe same-root move primitive.
28. Reject cross-volume moves.
29. Add per-action result model.
30. Add partial-failure semantics.

### Storage

31. Add SQLite storage crate.
32. Create migration framework.
33. Persist operation/action results.
34. Add history queries.

### Undo

35. Implement undo planning.
36. Block occupied original path.
37. Block changed destination restoration.
38. Journal undo operations.
39. Add adversarial undo fixtures.

### CLI

40. Add `tidyup scan`.
41. Add `tidyup plan`.
42. Add guided entry point.
43. Add preview and approval UX.
44. Add `tidyup apply`.
45. Add `tidyup history`.
46. Add `tidyup undo`.
47. Define exit-code contract.
48. Add JSON output schemas.

### Release

49. Package Windows binary.
50. Package macOS binary.
51. Add checksum generation.
52. Write install/uninstall documentation.
53. Add release checklist.
54. Publish `v0.1.0`.

---

## 41. Evidence Gates

Features should be considered complete only when objective evidence supports them.

Example: stale-plan protection.

```text
Implementation exists
    is NOT enough.

Required evidence:
- source modified after plan -> action rejected
- source replaced after plan -> action rejected
- destination created after plan -> action rejected
- no overwrite occurs
- operation result records skip reason
- Windows CI passes
- macOS CI passes
```

Example: undo.

```text
Required evidence:
- completed moves restore normally
- skipped apply actions are not undone
- failed apply actions are not undone
- occupied source path blocks restoration
- changed destination blocks unsafe restoration
- no overwrite occurs
- undo operation is journaled
- Windows CI passes
- macOS CI passes
```

---

## 42. Success Metrics

Primary success condition:

> A first-time user can install TidyUp, organize a disposable test folder, understand every proposed change, and undo the operation without consulting technical documentation.

Additional indicators:

- Zero known file-loss incidents.
- Safety bugs receive priority.
- Windows/macOS semantics remain consistent.
- Contributors add useful rule packs.
- Contributors can extend isolated modules.
- Users return for additional organization sessions.
- JSON/API compatibility remains predictable.
- Undo conflicts are safely surfaced instead of hidden.

---

## 43. Key Risks

### Filesystem edge cases

Mitigation:
Conservative defaults, platform adapters, large fixture matrix, failure injection.

### False confidence in undo

Mitigation:
Undo is validation-driven, not forced rollback.

### Too many automatic rules

Mitigation:
Deterministic rules, explanation, conflict surfacing, inspectable rule packs.

### Cross-platform inconsistency

Mitigation:
Windows/macOS CI from the beginning.

### Plugin security

Mitigation:
No executable plugin system in initial architecture.

### Scope expansion

Mitigation:
Do not simultaneously become a cleaner, backup utility, sync engine, and desktop manager.

### CLI accessibility

Mitigation:
Guided default workflow, examples, packaged binaries, eventual GUI only after core safety is proven.

---

## 44. Post-v0.1 Possibilities

Features may be considered only after the safe core is stable.

Potential areas:

- Recursive scan with explicit scope controls.
- Automatic but user-configurable collision naming.
- Cross-volume moves with crash-safe copy/verify/delete semantics.
- Trash/recycle-bin integration.
- Duplicate analysis.
- Large-file analysis.
- Age-based analysis.
- Rich TUI.
- Desktop application.
- File-type metadata beyond extensions.
- Community rule-pack registry.
- SDK bindings.
- Integration with external automation systems.

Any modifying future capability should reuse:

```text
plan -> validate -> approve -> execute -> journal -> undo
```

rather than introducing alternate mutation paths.

---

## 45. Portfolio Positioning

The project should be described technically as:

> A cross-platform local filesystem transaction-planning engine with deterministic rules, stale-state detection, guarded execution, append-only operation journaling, conflict-safe undo, versioned schemas, and Windows/macOS acceptance testing.

The strongest engineering themes are:

- Systems programming.
- Filesystem semantics.
- Cross-platform engineering.
- Domain modelling.
- Transaction/recovery design.
- Security boundaries.
- State persistence.
- CLI product design.
- Structured API design.
- Failure injection.
- Release engineering.
- Open-source governance.

The portfolio demo should include an intentional safety conflict, not only a happy-path sort.

Example:

```text
$ tidyup plan ./demo

12 files scanned
9 moves proposed
2 unsupported entries
1 collision

No files have been changed.

$ tidyup apply plan-0142

7 moved
1 skipped: destination created after preview
1 skipped: source changed after preview

Operation: op-0142

$ tidyup undo op-0142

6 restored
1 conflict: original path now contains another file

No existing files were overwritten.
```

This communicates the core engineering value more effectively than a simple “all files moved successfully” demonstration.

---

## 46. Definition of v0.1 Done

`v0.1.0` is complete only when all of the following are true:

### Product

- Guided flow works end to end.
- One root folder can be scanned.
- Direct-child files can be classified.
- Proposed moves are previewable.
- Approved files can be moved.
- Results are accurately reported.
- Completed actions can be safely undone.

### Safety

- No overwrite path exists in normal operation.
- Source staleness is detected.
- Destination changes are rechecked.
- Links are not followed.
- Root escape is rejected.
- Cross-volume moves are rejected.
- Partial operations are accurately recorded.
- Unsafe undo stops.

### Platform

- Supported Windows CI passes.
- Supported macOS CI passes.
- Unicode fixtures pass.
- Space-containing paths pass.
- Case-collision fixtures pass where applicable.
- Locked/permission scenarios are tested where reproducible.

### Developer experience

- Workspace builds from a clean clone.
- Architecture is documented.
- Core schemas are documented.
- Contribution instructions work.
- Rule-pack examples exist.
- Beginner issues exist.

### Distribution

- Windows artifact exists.
- macOS artifact exists.
- Checksums exist.
- Installation instructions exist.
- Uninstall/data-location instructions exist.
- Known limitations are explicit.

---

## 47. Immediate Implementation Order

The first implementation sequence should be:

1. Initialize the Cargo workspace.
2. Create `tidyup-core`, `tidyup-platform`, `tidyup-storage`, `tidyup-cli`, and `tidyup-testkit`.
3. Add CI on Windows and macOS.
4. Write the core safety invariant tests.
5. Implement disposable filesystem fixtures.
6. Define typed domain identifiers and `FileSnapshot`.
7. Implement read-only scanner.
8. Define rule schema v1.
9. Implement built-in extension classification.
10. Implement plan generation.
11. Implement root-containment and collision validation.
12. Add human-readable and JSON plan output.
13. Implement execution-time revalidation.
14. Add safe same-root file movement.
15. Implement SQLite operation journaling.
16. Implement guarded undo.
17. Connect the guided CLI.
18. Package clean Windows/macOS builds.
19. Complete open-source documentation.
20. Release the narrow MVP.

No advanced feature should interrupt this sequence unless it is required to satisfy a safety or portability invariant.

---

## 48. Final Architectural Invariants

These statements should be treated as project-level invariants:

1. **Scanning never mutates user files.**
2. **A plan is not execution.**
3. **Every modifying action is revalidated immediately before execution.**
4. **TidyUp does not overwrite existing files.**
5. **v0.1 destinations remain inside the selected root.**
6. **Imported rules and plans are untrusted input.**
7. **Link-like entries are not followed implicitly.**
8. **Partial failure is recorded as partial failure.**
9. **Undo reverses only actions that actually completed.**
10. **Undo never overwrites conflicting current data.**
11. **Operation history is append-oriented and auditable.**
12. **Windows and macOS behavior are tested continuously.**
13. **The CLI and any future GUI use the same core engine.**
14. **Community extensibility begins with declarative, inspectable rules.**
15. **Deletion is outside the initial release.**
16. **Privacy-preserving local operation is the default.**
17. **Safety claims require verification evidence, not only implementation claims.**

---

## 49. One-Sentence Project Definition

> **TidyUp is an open-source, local-first filesystem planning engine and guided organizer that lets users preview, validate, execute, audit, and safely undo deterministic file moves across Windows and macOS.**

