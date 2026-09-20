# One unreadable pulled object is reported and skipped, and the pull lands

## Context

A pull brings back objects a remote built. `dam` did not build them and cannot assume it can read
every one: a helper may send a field value `dam` has no vocabulary for, or an object of a shape a
later protocol version added.

## Decision

An object `dam` cannot convert is recorded as a `PullFailed` notice naming its remote id and the
reason, and the rest of the pull lands.

## Consequence

The operator sees the skipped object in `dam status` rather than losing a whole pull to one bad
record. The pull is still atomic against a failure: every record family it does write lands in one
unit of work, so a refused pull leaves none of its own notices behind.

The alternative, failing the pull, hands one malformed upstream record the power to stop `dam`
syncing at all, which a remote `dam` does not control should not have.

Where the code says it: the rejected loop in `crates/dam-application/src/use_cases/pull.rs`.
