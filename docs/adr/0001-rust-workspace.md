# ADR-0001: Rust Workspace Boundaries

## Status

Accepted on August 18, 2026.

## Context

TidyUp needs early separation between core safety logic, platform-sensitive filesystem behavior, persistence, CLI interaction, and reusable test infrastructure. The project specification already describes a multi-crate Rust workspace as the intended architecture.

Without explicit boundaries, the CLI could absorb mutation logic, platform-specific edge cases could leak into domain code, and later safety work would become harder to test or review.

## Decision

TidyUp uses a Rust workspace with distinct crates for:

- `tidyup-core`
- `tidyup-platform`
- `tidyup-storage`
- `tidyup-cli`
- `tidyup-testkit`

Core safety and planning rules belong in `tidyup-core`. OS-sensitive filesystem behavior belongs in `tidyup-platform`. Persistent operation history belongs in `tidyup-storage`. User interaction belongs in `tidyup-cli`. Disposable fixtures and acceptance helpers belong in `tidyup-testkit`.

## Consequences

- Safety-critical logic can be tested without terminal or OS UI concerns.
- Platform behavior has an explicit place to evolve for Windows and macOS differences.
- CLI features must call shared core logic instead of reimplementing execution paths.
- Storage choices can change with less impact on planning or UI code.
- Contributors get clearer ownership boundaries for later issues.
