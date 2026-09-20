# A push retry resends the committed state, not the stale commit change

## Context

Delivery to a remote is at least once. `dam` cannot tell a request that never arrived from an answer
that never came back, so an interrupted or partly failed push leaves objects owed a retry. The
question is what the retry sends: the change as the commit recorded it, or the object as it stands
now in the committed view.

An object can have moved on since the commit that first tried to push it. Sending the recorded
change would push a state the operator has already replaced, and the remote would then hold
something `dam` no longer believes.

## Decision

A retry sends a synthetic update built from the committed view, except where an unpushed commit
already covers that oid. A coalesced commit change already ends at the committed state, so leaving
it in place resends exactly what the synthetic one would have.

## Consequence

The commit that names the change is kept, which is what lets a resend after an interrupted push
carry the same idempotency key the interrupted attempt carried. A remote that deduplicates by key
therefore does the work once.

Where the code says it: `changes_to_send` in `crates/dam-application/src/use_cases/push.rs`.
