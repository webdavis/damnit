# An unanswered mutation is a failure, not a silence

## Context

A helper answers a push with one result per mutation. A helper may answer fewer than it was sent:
it crashed part way, it lost track, or it simply does not report every one. `dam` has to decide what
an oid with no result means.

## Decision

A sent mutation with no answer at all, not even an explicit failure, is treated exactly like a
reported failure. It gets a `PushFailed` notice, it goes on the retry list, and the commit that
carried it is not marked pushed.

## Consequence

The decision is fail-closed. `dam` would rather send a mutation twice, which the idempotency key
makes safe against a remote that deduplicates, than record work as delivered when it never reached a
known state on the remote. The alternative, dropping the oid silently, loses the operator's change
with no trace in `dam status`.

Where the code says it: the unanswered loop in `crates/dam-application/src/use_cases/push.rs`.
