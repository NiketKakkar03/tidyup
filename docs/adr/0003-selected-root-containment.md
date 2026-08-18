# ADR-0003: Selected-Root Containment

## Status

Accepted on August 18, 2026.

## Context

The `v0.1` scope is intentionally narrow: organize one selected root and move files only into subfolders inside that root. Paths originating from rule packs, plans, or platform-specific path handling can become dangerous if containment is not enforced consistently.

Containment also protects against accidental broad filesystem changes and malicious or malformed inputs that attempt destination escape.

## Decision

Every planned and executed destination in `v0.1` must remain inside the user-selected root.

Containment is enforced during planning, validation, execution, and undo-related checks. Imported plans and rule packs do not bypass this rule. Link-like entries and path tricks that could redirect work outside the selected root are rejected or skipped.

## Consequences

- The mutation surface remains intentionally small for the MVP.
- Destination validation must be platform-aware, especially for case rules and link-like behavior.
- Rule-pack design stays focused on categorization, not arbitrary destination authority.
- Future features that move data outside the selected root require a new ADR and explicit scope expansion.
