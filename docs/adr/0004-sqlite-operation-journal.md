# ADR-0004: SQLite Operation Journal

## Status

Accepted on August 18, 2026.

## Context

TidyUp needs durable local history for apply and undo operations. The journal must record partial success truthfully, support later history queries, and provide enough information to drive guarded undo behavior.

Flat files could work for small prototypes, but schema evolution, atomic updates, and queryability matter early because the journal is part of the safety architecture rather than a convenience log.

## Decision

TidyUp uses local SQLite storage as the initial operation journal for `v0.1`.

The SQLite journal stores operation metadata, per-action outcomes, and the linkage needed for undo planning and auditability. History is append-only at the operation level: undo creates a new operation record rather than rewriting prior history.

## Consequences

- Durable local history supports truthful status reporting and later undo.
- Schema versioning and migrations become explicit parts of storage work.
- The storage crate can encapsulate persistence concerns without leaking them into the core domain.
- Contributors have a clear baseline for tests and for future history-oriented features.
