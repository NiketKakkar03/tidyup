# ADR-0005: Declarative Rules Only

## Status

Accepted on August 18, 2026.

## Context

TidyUp wants community extensibility, but executable plugins or embedded scripts would expand the trust boundary too early and make safety review much harder. Rule logic in `v0.1` should remain deterministic, inspectable, and easy to validate before use.

Plans and rule packs are already treated as untrusted input, so extension points must be narrow and auditable.

## Decision

Community extension in `v0.1` is limited to validated declarative rule packs.

Rule packs may describe deterministic classification behavior, but they may not execute arbitrary code, invoke scripts, or bypass planner and validator guarantees. Built-in and imported rules are subject to the same validation and containment constraints.

## Consequences

- Extensibility stays compatible with a safety-first review process.
- Rule-pack parsers and validators become important enforcement points.
- Contributors can share inspectable rules without expanding runtime execution privileges.
- Future executable extension models require a separate design decision and security review.
