# A task removed in the app with writes owed owes nothing

## Context

A push whose answer was lost keeps its writes owed. When Todoist then reports the task deleted in
the app, the next push would send what it still owed, and with no Todoist id that write is a create.
The task the person deleted would come back.

## Decision

The pull clears what `dam` owed the task, keeps it here with the removed-upstream notice, and stores
its committed state as the remote snapshot of an object with no remote id. It also drops the push
failed notice for that task, since nothing is owed any more. It does this for every task removed in
the app that is still committed here, whether or not a write was owed, so no source of owed writes
needs a case of its own.

`LastSeen::RemovedInTheAppNothingOwed` names that state. `LastSeen::removed_in_the_app_nothing_owed`
writes it. The push reads it through `LastSeen::read` for every object it would send, and declares
such an object only by what changed since that snapshot. Nothing is sent for it until the person
changes it here, and then it goes as one create.

Every use case reads and writes the remote snapshot through `LastSeen`, and only `LastSeen::Held`
means the remote still holds the object. A push that asks what is still upstream under a parent it
deletes, or what to move out of it, sees only held objects.
`scripts/check-remote-snapshot-readers.sh`, run by `just gates`, fails when any other use-case file
touches the snapshot.

When the record holds a delete, the removal is that delete's answer. When the task is no longer
committed here, both sides agree it is gone and nothing is kept. In both cases, and when the task
is removed from the working copy but not yet committed, the pull shows no notice and counts the
removal as a delete confirmed.

## Consequence

The existing tables hold the state, with no migration. A table of its own would name the state in
the store, but it needs a migration, a repository, and a way for the push to tell changes made
before the mark from changes made after it. A commit is marked pushed whole and often holds other
objects, so that split has no clean place.

The push reads the state for every object, not only for those on its retry list. The retry list is
replaced by each push, while the commit that holds the task's last change can stay unpushed because
of another object in it. A later push would then send that change as a create.

The person who wants the task back in Todoist edits it. A helper that accepts a create and names no
id also leaves a snapshot with no remote id. `LastSeen::read` reads it as the same state, and the
push then declares only what changed since that create, which is a create again. A push also no
longer counts such a task as upstream under a parent it deletes.

Where the code says it: `crates/dam-application/src/use_cases/last_seen.rs`,
`crates/dam-application/src/use_cases/pull/removed.rs` and
`crates/dam-application/src/use_cases/push/owed.rs`.
