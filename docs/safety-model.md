# TidyUp Safety Model

## Status

Accepted baseline for MVP issue #3 on August 18, 2026.

## Purpose

This document captures the safety invariants that all `v0.1` implementation work must preserve. It is the repo-level source of truth for safety-sensitive behavior until more detailed implementation docs exist.

## Scope

These invariants apply to:

- scanning
- planning
- validation
- execution
- journaling
- undo
- built-in and imported rule packs
- imported plans

They apply across Windows and macOS for the `v0.1` product boundary: one selected root, direct-child file organization, and guarded undo.

## Core Safety Invariants

1. Read-only stages stay read-only.
   Scanning, classification, plan generation, and plan validation must not mutate user data.
2. A plan is a proposal, not permission.
   A saved or approved plan does not bypass execution-time checks.
3. Every modifying action is revalidated immediately before execution.
   Source state, destination state, and containment assumptions must be checked again at apply time and undo time.
4. TidyUp never overwrites an existing file.
   This applies during both apply and undo. Collisions are surfaced and stopped rather than resolved by silent replacement or auto-rename.
5. All planned destinations stay inside the user-selected root.
   `v0.1` does not move files outside the selected root and does not allow rule packs or imported plans to do so indirectly.
6. Link-like entries are not followed implicitly.
   Symbolic links, junctions, reparse points, aliases, and similar entries are skipped or rejected according to platform support.
7. Plans and rule packs are untrusted input.
   Imported configuration and persisted plans must be parsed, validated, and constrained before use.
8. Undo is guarded, not privileged.
   Undo restores only actions that still validate safely in the current filesystem state and must stop before overwriting newer data.
9. History must remain truthful.
   Partial success, skips, and failures are recorded explicitly so the journal reflects what actually happened.
10. Safety claims require durable evidence.
   Later implementation work should add tests for each invariant where technically feasible.

## Trust Boundaries

Treat the following as untrusted input:

- selected root paths provided by users
- imported plans
- imported rule packs
- filesystem state observed during scan and revalidation
- platform-specific path semantics and case behavior

The system must defend against:

- stale plans caused by filesystem changes after planning
- destination escape outside the selected root
- collisions that would overwrite existing files
- link-assisted redirection
- malformed or ambiguous rule definitions
- undo requests that would restore data into a conflicting current path

## ADR Baseline

The following ADRs define the current architectural baseline for these invariants:

- [ADR-0001: Rust workspace boundaries](adr/0001-rust-workspace.md)
- [ADR-0002: No-overwrite policy](adr/0002-no-overwrite-policy.md)
- [ADR-0003: Selected-root containment](adr/0003-selected-root-containment.md)
- [ADR-0004: SQLite operation journal](adr/0004-sqlite-operation-journal.md)
- [ADR-0005: Declarative rules only](adr/0005-declarative-rules-only.md)

## How To Use This Document

- Reference this file when adding or reviewing any filesystem mutation behavior.
- Reference the ADRs when a change touches architecture, storage, rule extensibility, or execution guarantees.
- Reject implementation shortcuts that weaken these invariants, even if they simplify tests or demos.
