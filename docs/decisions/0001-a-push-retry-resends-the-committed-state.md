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
it in place resends exactly what the synthetic one would have. An object removed since the failed
attempt is resent as a removal.

## Consequence

A resend repeats its idempotency key when nothing about the object was answered since and the same
thing is sent. A key is made from the object and the count of answers the remote gave about it.
The count moves only on an answer the remote gave: an accepted write, or a refusal from the remote.
An unanswered write does not move it, and neither does a refusal the remote gave no verdict on,
which the helper marks `unasked`. An update's key also takes a hash of the declared fields with
their values as the protocol puts them on the wire, in field order, and the container it moves
into. A create's and a delete's take no content, so an edit between an unanswered create and its
resend keeps its key, and a remote that runs a key once runs the create once. No key comes from a
commit.

The fields an unanswered write declared stay in doubt until a write declaring them is accepted. The
next push declares them with its own difference, even when nothing else is owed. A push stopped
before any answer stores only this doubt, for every write it handed to the helper. A create accepted
while fields are in doubt may have been skipped as already run, so the same push follows it with an
update of those fields. A create in doubt whose object is removed here before its resend leaves
nothing to send. The push then leaves a notice that the remote may hold a copy, and stops holding
the object in doubt.

Where the code says it: `changes_to_send` in `crates/dam-application/src/use_cases/push/owed.rs`,
`key` in `push/idempotency.rs`, and `settle_doubt` in `push/doubt.rs`.
