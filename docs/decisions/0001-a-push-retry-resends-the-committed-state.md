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

Every object a push owes is sent from what the remote snapshot holds to the committed state, whether
an unpushed commit, a retry or a recorded write makes it owed. The changes an unpushed commit
recorded only say which objects are owed. A commit left unpushed by one unanswered write can hold an
older value of another object that a later push already sent, so its recorded change is never sent.
An object removed since the failed attempt is resent as a removal.

## Consequence

Every write handed to a helper is recorded, per remote and object, before the call: its key, its
operation, the container it moves into, and each declared field with its value as the protocol puts
it on the wire. A create declares every field the remote takes. The record holds only writes with no
answer yet. An accepted write clears it, a refusal drops the write it refused, a refusal the remote
gave no verdict on drops the write only if this push added it, and no answer leaves it.

A key comes from the object and the count of answers the remote gave about it. The count moves on an
accepted write and on a refusal from the remote, not on an unanswered write or on a refusal marked
`unasked`. An update's or delete's key also takes its place in the record, and an update's the
container and declared values. A write that sends what the last recorded write sent repeats its key
and adds nothing to the record. Any other write is added and gets a key of its own, even when it
sends what an older recorded write sent. A create's key takes no content and no place, so an edit
between an unanswered create and its resend keeps the key, and a remote that runs a key once runs
the create once. No key comes from a commit.

Each write also declares every field where a recorded write sent another value than the object
holds now. A create accepted after a recorded write that sent other values may have been skipped as
already run, so the same push follows it with an update of those fields only.

A create with no answer whose object is removed here, or no longer goes to that remote, leaves a
notice that the remote may hold a copy. The record forgets it and the count moves, so a create that
comes back is a new request. A record with no create, for an object the remote no longer holds, is
forgotten with no notice. An object the remote holds keeps its record when it leaves the remote's
path, and the record is declared when it comes back. Its record is forgotten when nothing is left to
declare and it holds no delete, or when the object is removed here and no write goes to the remote.
The count moves whenever a record is forgotten, so a later write never repeats a key the remote may
have run.

Where the code says it: `changes_to_send` in `crates/dam-application/src/use_cases/push/owed.rs`,
`key` in `push/idempotency.rs`, `attempting` in `push/attempting.rs`, and `settle_attempts` in
`push/attempts.rs`.
