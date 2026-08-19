# Security Policy

## Reporting

If you believe TidyUp has a security or data-safety issue, please avoid filing a public exploit description first.

Report details to the maintainers through a private GitHub security advisory or a private maintainer contact path once one is published.

## Scope

Security for TidyUp includes classic security concerns and data-safety concerns such as:

- unintended overwrite behavior
- escaping the selected root
- unsafe undo behavior
- trust boundary issues in plans, journals, or rule packs

## Supported Version

As of August 19, 2026, the active development target is `main` toward `v0.1.0`.

## Threat Model Summary

- plans are untrusted inputs
- rule packs are untrusted inputs
- journal contents are untrusted persisted inputs
- filesystem state may change between planning and execution

See [docs/SAFETY_MODEL.md](/Users/niketkakkar/.codex/worktrees/a633/tidyup/docs/SAFETY_MODEL.md) for the current safety model.
