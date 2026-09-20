# A failed blocker read refuses the completion rather than reading as closed

## Context

Completing a task checks whether its dependencies and children are still open. Each check is a store
read. A read can fail for reasons that have nothing to do with the object: a busy store, a corrupt
row, a full disk.

A missing object is a different case. An object that is not there cannot be open, so a missing
dependency reads as closed.

## Decision

A missing object reads as closed. A read that fails propagates as an error, so the completion is
refused.

## Consequence

This is the fail-closed direction on the rule the tool exists to keep. Treating a failed read as
closed would let a blocked task complete because the store hiccupped, which is the one outcome the
blocker rule is there to prevent. The cost is that a transient store failure refuses a completion
the operator could have had, which they see and can retry.

Where the code says it: `is_open` in `crates/dam-application/src/use_cases/complete.rs`.
