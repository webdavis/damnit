# A pull holds uncommitted work the app changed as a conflict

## Context

A pull compared each pulled object with the working copy only. An object removed here and not yet
committed looked new, so the pull created it again under its old oid and committed it. An edit that
was not committed refused the whole pull, and no other object landed until the person committed or
reset it.

## Decision

`Untouched::of` in `crates/dam-application/src/use_cases/pull/untouched.rs` decides it for every
object. When the working copy or the stage holds work the commit does not, and the remote changed
the object since `dam` last saw it, the pull does not touch the object. It marks a conflict, and
every other object still lands. When the remote did not change the object, the work is left alone
with no conflict. When the remote already holds what the working copy holds, the pull takes it as
usual.

A conflict over an object removed here has no side of ours. `dam status` prints it as
`ours: removed`, and `--json` gives `ours` as null. `dam resolve --ours` commits the removal, and
`--theirs` brings back what the remote holds and commits it. The store keeps that side as the JSON
document `null`, so the conflicts table needs no migration.

## Consequence

The `dirty_on_pull` refusal is gone. A client that matched on it now sees a pull that succeeds and
counts the object among its conflicts.
