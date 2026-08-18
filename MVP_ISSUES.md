# TidyUp MVP Issue Backlog

This backlog turns the specification in [TIDYUP_PROJECT_STRUCTURE.md](/Users/niketkakkar/Desktop/Projects/tidyup/TIDYUP_PROJECT_STRUCTURE.md) into a practical sequence of issues to open and execute one by one toward the `v0.1.0` MVP.

Scope guard for this backlog:

- Target only the `v0.1` promise: a first-time user can safely organize one folder and undo the result.
- Favor end-to-end slices over micro-issues when the work is tightly coupled.
- Keep all destinations inside the selected root.
- Do not add post-MVP features such as recursive organization, deletion, duplicate cleanup, cross-volume moves, or a desktop GUI.

## How To Use This Backlog

Create these issues in order. Each issue assumes the previous one is complete unless noted otherwise.

Suggested labels:

- `mvp`
- `area:core`
- `area:platform`
- `area:storage`
- `area:cli`
- `area:testing`
- `area:docs`
- `safety`

---

## Issue 1: Initialize the Rust workspace and crate boundaries

**Goal**
Create the repository skeleton for the multi-crate architecture: `tidyup-core`, `tidyup-platform`, `tidyup-storage`, `tidyup-cli`, and `tidyup-testkit`.

**Acceptance checklist**

- [ ] Root `Cargo.toml` defines a workspace with the five initial crates.
- [ ] Each crate builds with a minimal library or binary entry point.
- [ ] The workspace layout matches the structure described in the spec closely enough to avoid early churn.
- [ ] The CLI crate depends on the core through crate boundaries rather than inline implementation.
- [ ] `cargo build` succeeds from a clean clone on the maintainer machine.

**Notes**
This is the structural foundation for every later issue.

---

## Issue 2: Add formatting, linting, and baseline cross-platform CI

**Goal**
Set up the quality and verification loop before adding real logic.

**Acceptance checklist**

- [ ] Rust formatting and linting configuration files exist at the repo root.
- [ ] A GitHub Actions workflow runs build, format check, lint check, and tests.
- [ ] CI includes both Windows and macOS runners.
- [ ] CI is wired to the workspace rather than a single crate.
- [ ] Failing checks produce actionable output for contributors.

**Notes**
Cross-platform correctness is a product feature, so CI belongs at the start.

---

## Issue 3: Capture the safety invariants and ADR baseline

**Goal**
Turn the spec’s safety model into durable repo documentation that engineering work can reference.

**Acceptance checklist**

- [ ] Core safety invariants are documented in a dedicated doc.
- [ ] ADR-0001 through ADR-0005 are added.
- [ ] The no-overwrite, selected-root containment, SQLite journal, and declarative-rules decisions are explicit.
- [ ] The docs explain that plans and rule packs are untrusted input.
- [ ] Later implementation issues can reference these docs as the source of truth.

**Notes**
This issue keeps safety decisions from drifting during implementation.

---

## Issue 4: Build the filesystem testkit and acceptance fixture scaffolding

**Goal**
Create the disposable test infrastructure needed to prove behavior before file mutation work lands.

**Acceptance checklist**

- [ ] `tidyup-testkit` can create disposable directory fixtures.
- [ ] Fixtures support filenames with spaces and Unicode.
- [ ] Fixtures can represent collisions and unsupported link-like entries where platform-supported.
- [ ] The repo includes acceptance-test scaffolding for the observable `scan -> plan -> validate -> apply -> undo` flow.
- [ ] The initial tests are executable, even if they begin as pending or partial behavior specifications.

**Notes**
The completion gate is that observable behavior is represented in tests before real mutation exists.

---

## Issue 5: Define the core domain types, identifiers, and `FileSnapshot`

**Goal**
Introduce the typed models that later stages will share.

**Acceptance checklist**

- [ ] Core typed identifiers exist for plans, operations, and actions.
- [ ] `FileSnapshot` captures the metadata needed for scan, validation, and undo.
- [ ] Domain types live in `tidyup-core` and remain platform-neutral where possible.
- [ ] Serialization boundaries are considered for future JSON and journal use.
- [ ] Unit tests cover the basic invariants of the new domain types.

**Notes**
This is the point where the shared language of the engine becomes concrete.

---

## Issue 6: Implement the read-only direct-child scanner

**Goal**
Scan exactly one selected root, without mutating anything, and report supported versus skipped entries.

**Acceptance checklist**

- [ ] The scanner inspects one user-selected root directory.
- [ ] Only direct child entries are considered for `v0.1`.
- [ ] Read-only scanning never mutates user files.
- [ ] Unsupported or risky entries are skipped with structured reasons.
- [ ] Link-like entries are detected and not followed implicitly.
- [ ] Scanner behavior is covered by fixture-based tests.

**Notes**
This should satisfy the first real observable user behavior in the spec.

---

## Issue 7: Define rule-pack schema v1 and built-in extension rules

**Goal**
Create the first deterministic classification system for `v0.1`.

**Acceptance checklist**

- [ ] Rule-pack schema v1 is defined and documented.
- [ ] Built-in extension-based classification rules exist for the default pack.
- [ ] Rule validation rejects malformed or ambiguous definitions.
- [ ] Rule evaluation is deterministic for the same input state.
- [ ] Classification results include enough rule identity to explain later moves.

**Notes**
This issue should stay narrow: metadata-based classification only, no AI or content analysis.

---

## Issue 8: Implement the planner and plan schema v1

**Goal**
Convert scan and classification results into a previewable move plan.

**Acceptance checklist**

- [ ] `Plan` and `PlannedMove` are defined in the core domain.
- [ ] Planned destinations are subfolders inside the selected root only.
- [ ] Existing destination collisions are detected without overwriting anything.
- [ ] Same-plan collisions are detected deterministically.
- [ ] Invalid destination names are rejected.
- [ ] Plan schema v1 can be serialized for later preview and storage.

**Notes**
A plan is a proposal, not permission to mutate.

---

## Issue 9: Add read-only plan validation and stable validation reason codes

**Goal**
Validate plans before execution and establish a stable reason taxonomy for rejections.

**Acceptance checklist**

- [ ] Source snapshot comparison logic exists.
- [ ] Destination preflight checks exist for invalid or conflicting targets.
- [ ] Stable validation reason codes are defined for machine-readable output.
- [ ] Stale-plan regression fixtures exist for changed source and changed destination cases.
- [ ] Validation results distinguish valid, skipped, and rejected actions clearly.

**Notes**
This issue prepares the transition from preview to guarded execution.

---

## Issue 10: Add `tidyup scan` and `tidyup plan` with human and JSON output

**Goal**
Expose the read-only engine through usable CLI commands.

**Acceptance checklist**

- [ ] `tidyup scan` is implemented.
- [ ] `tidyup plan` is implemented.
- [ ] Human-readable output explains what was scanned, skipped, and proposed.
- [ ] Stable JSON output exists for read-only commands.
- [ ] CLI output makes it clear that no files were changed during scan or plan.
- [ ] Command behavior is covered by acceptance or snapshot-style tests.

**Notes**
This is the end of the first public read-only milestone.

---

## Issue 11: Implement execution-time revalidation and the safe same-root move primitive

**Goal**
Add the guarded mutation path while preserving the core safety invariants.

**Acceptance checklist**

- [ ] Every approved action is revalidated immediately before execution.
- [ ] Same-root file moves are implemented through a dedicated platform-aware primitive.
- [ ] Cross-volume moves are rejected.
- [ ] No overwrite path exists during apply.
- [ ] Changed source and newly occupied destination cases are skipped safely.
- [ ] Tests prove that stale-source and destination-collision scenarios do not overwrite data.

**Notes**
This is the most safety-sensitive implementation issue so far.

---

## Issue 12: Add per-action execution results and partial-failure semantics

**Goal**
Make apply results precise and auditable instead of all-or-nothing.

**Acceptance checklist**

- [ ] The executor returns per-action result records.
- [ ] Completed, skipped, and failed actions are represented distinctly.
- [ ] Partial success is never reported as full success.
- [ ] Result records preserve the reason for skipped or failed actions.
- [ ] Acceptance tests cover mixed-result operations.

**Notes**
Accurate partial-failure reporting is one of the product principles.

---

## Issue 13: Add SQLite-backed operation journaling and history queries

**Goal**
Persist operation history durably so apply and undo can be audited.

**Acceptance checklist**

- [ ] `tidyup-storage` owns SQLite-backed persistence.
- [ ] A migration framework exists.
- [ ] Operation records and per-action results are persisted.
- [ ] Journal entries are append-oriented and auditable.
- [ ] History queries can retrieve prior operations needed for CLI display and undo.
- [ ] Storage tests cover migration startup and basic persistence flows.

**Notes**
The journal is a core MVP capability, not a later observability extra.

---

## Issue 14: Add `tidyup apply` and approval-oriented CLI flow

**Goal**
Deliver the first end-to-end organizing command that a user can safely review and run.

**Acceptance checklist**

- [ ] `tidyup apply` is implemented.
- [ ] The CLI includes approval handling before mutation.
- [ ] Users can review a summary and proposed moves before execution.
- [ ] Apply output shows exact counts and reasons for moved, skipped, and failed actions.
- [ ] Successful apply operations are written to history.
- [ ] Acceptance tests cover a happy path plus at least one intentional safety conflict.

**Notes**
This is the point where the project first fulfills the “organize one folder safely” half of the MVP.

---

## Issue 15: Implement guarded undo planning, validation, and execution

**Goal**
Allow safe reversal of completed operations without pretending rollback is unconditional.

**Acceptance checklist**

- [ ] Undo plans are derived only from actions that actually completed.
- [ ] Undo revalidates current filesystem state before restoration.
- [ ] Occupied original paths block unsafe restoration.
- [ ] Changed destination state blocks unsafe restoration.
- [ ] Undo never overwrites conflicting current data.
- [ ] Undo results are journaled as first-class operations.
- [ ] Adversarial undo fixtures cover normal restore and blocked restore cases.

**Notes**
Undo is validation-driven, not forced rollback.

---

## Issue 16: Add `tidyup history`, `tidyup history show`, and `tidyup undo`

**Goal**
Complete the user-facing recovery loop for the MVP.

**Acceptance checklist**

- [ ] `tidyup history` lists prior operations clearly.
- [ ] `tidyup history show` displays enough detail to audit one operation.
- [ ] `tidyup undo` restores safe actions from a chosen prior operation.
- [ ] CLI recovery output explains conflicts instead of hiding them.
- [ ] Exit codes distinguish success, partial success, and blocked undo cases where appropriate.
- [ ] Acceptance tests cover the full apply-then-undo loop.

**Notes**
This completes the user-visible MVP workflow promised in the spec.

---

## Issue 17: Harden platform fixtures and cross-platform edge-case coverage

**Goal**
Raise confidence that the MVP works consistently on Windows and macOS, including awkward filesystem cases.

**Acceptance checklist**

- [ ] Windows CI passes for the supported matrix.
- [ ] macOS CI passes for the supported matrix.
- [ ] Unicode filename fixtures pass.
- [ ] Space-containing path fixtures pass.
- [ ] Case-collision scenarios are tested where applicable.
- [ ] Locked or permission-sensitive scenarios are tested where reproducible.
- [ ] Known platform limitations are documented where tests cannot be made identical.

**Notes**
This issue is the evidence gate for many earlier safety claims.

---

## Issue 18: Complete MVP documentation and contributor on-ramp

**Goal**
Make the repository usable by an outside developer and understandable by a first-time user.

**Acceptance checklist**

- [ ] `README.md` explains the product, scope boundaries, install path, and guided workflow.
- [ ] `CONTRIBUTING.md` explains how to build, test, and contribute safely.
- [ ] `SECURITY.md`, `GOVERNANCE.md`, and `CODE_OF_CONDUCT.md` exist.
- [ ] Architecture, safety model, journal format, plan format, and CLI contract docs exist.
- [ ] Rule-pack examples and documentation exist.
- [ ] `ROADMAP.md` reflects the MVP and explicitly lists out-of-scope items.
- [ ] Beginner-friendly issue labels or starter tasks are documented.

**Notes**
This is part of the MVP definition of done, not a post-release polish task.

---

## Issue 19: Package Windows and macOS artifacts with install/uninstall docs

**Goal**
Prepare the project for practical MVP distribution.

**Acceptance checklist**

- [ ] A Windows binary artifact is produced by the release process.
- [ ] A macOS binary artifact is produced by the release process.
- [ ] Checksums are generated for release artifacts.
- [ ] Installation instructions are documented.
- [ ] Uninstall and local data-location instructions are documented.
- [ ] Known limitations are explicit in release-facing docs.

**Notes**
Keep packaging simple and reliable; avoid expanding scope into installers unless clearly needed.

---

## Issue 20: Run the `v0.1.0` release checklist and publish the MVP

**Goal**
Ship the narrow MVP with evidence that it meets the specification.

**Acceptance checklist**

- [ ] A release checklist exists and is followed.
- [ ] The end-to-end guided flow works on a disposable demo folder.
- [ ] The demo includes at least one intentional safety conflict, not only a happy path.
- [ ] Release notes summarize capability, safety guarantees, and known limitations.
- [ ] `CHANGELOG.md` includes the `v0.1.0` entry.
- [ ] The maintainers agree the MVP satisfies the documented definition of done.

**Notes**
Do not pull M6+ features into this issue. Ship the narrow promise first.

---

## Non-MVP backlog that should stay out of this sequence

- Recursive directory organization
- Deletion or trash integration
- Duplicate cleanup
- Large-file or old-file analysis
- Cross-volume copy/verify/delete behavior
- Automatic collision renaming
- GUI or desktop app
- Executable plugin system
- AI or content-based classification
- Scheduled or background cleanup

## Recommended GitHub issue creation order

1. Issue 1 through Issue 4: foundation and evidence scaffolding
2. Issue 5 through Issue 10: read-only engine and preview UX
3. Issue 11 through Issue 14: safe execution and journaling
4. Issue 15 through Issue 16: undo and recovery loop
5. Issue 17 through Issue 20: hardening, docs, packaging, release
