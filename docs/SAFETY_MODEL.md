# Safety Model

TidyUp is built around a few non-negotiable rules:

- no silent overwrite
- selected-root containment
- preview before mutation
- revalidation immediately before execution
- undo is conditional, not forced rollback

Important trust boundaries:

- rule packs are untrusted inputs
- plans are untrusted inputs
- history/journal contents are untrusted persisted inputs

Execution safety:

- only direct-child source files are considered in `v0.1.0`
- moves stay inside the selected root
- an occupied destination blocks the action
- a changed or missing source blocks the action

Undo safety:

- only completed actions from the original operation are candidates
- the current filesystem state is revalidated before restore
- an occupied original path blocks restore
- a changed/missing moved file blocks restore
