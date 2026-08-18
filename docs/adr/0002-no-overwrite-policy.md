# ADR-0002: No-Overwrite Policy

## Status

Accepted on August 18, 2026.

## Context

The product promise depends on users being able to trust that TidyUp will not destroy or replace existing data while organizing files or undoing prior moves. Destination collisions can appear during planning, between planning and execution, or during undo after the filesystem has changed again.

Automatic overwrite and automatic collision renaming would hide risk instead of surfacing it clearly.

## Decision

TidyUp never overwrites an existing path during apply or undo.

If a destination path is already occupied, the action must be rejected or skipped with an explicit reason. The planner, validator, executor, and undo flow all enforce this invariant independently.

`v0.1` also does not auto-rename files to resolve collisions silently.

## Consequences

- Users can inspect conflicts without fear of silent data loss.
- Stale-plan and undo revalidation remain meaningful safety gates.
- Later implementation must model collisions as first-class outcomes, not incidental errors.
- Some actions will stop instead of completing automatically, but the result is safer and easier to audit.
