# A push sends what holds an object before the object, and removes it after

## Context

A helper sends each mutation as its own request, in the order it receives them. A remote ties some
objects to others: Todoist files a task under a project, a section or a parent task, and deleting a
project, section or task deletes everything still under it. `dam` used to send owed changes in oid
order, which is random. Removing a parent and moving its child up in one commit could send the
parent's delete first, and the remote then deleted the child. A child created in the same push as
its parent could likewise arrive before the parent existed and be refused.

## Decision

A push sends every create and update before any delete. Creates and updates go outermost first, by
the depth of the path the object ends up at. Deletes go innermost first, by the depth of the path the
object was at. Changes at the same depth keep oid order.

A delete is held until the remote holds nothing under the deleted object. `dam` reads where the
remote holds each object from its remote snapshot, so a push sends in waves: each wave holds every
change whose blockers the remote has accepted, then the next wave reads the outcomes. A delete whose
blocker was refused, held or never sent fails with a reason naming that object and goes on the retry
list. An object moved outside the remote's `path` is never sent, so it holds the delete until the
person moves it back inside. A delete also waits while this push still has a create or a move that
files an object under the deleted object, and once that lands it holds the delete like any other.
Decision 0012 covers a delete held only because two objects share a folder.

## Consequence

An object's container exists before the object is created or moved into it, and an object is
removed only after everything that was under it has moved out or been removed. The order comes from
the path, so it is the same for every remote and every retry. Idempotency keys depend only on the
commit and the oid, so reordering does not change them.

A push can still stop between two requests. Each request is settled on its own, so the next push
sends only what is still owed, in the same order. When the helper stops a later wave, as on a rate
limit, the earlier waves are still settled and nothing of the later wave counts as sent.

Where the code says it: `crates/dam-application/src/use_cases/push/sending_order.rs` and
`crates/dam-application/src/use_cases/push/removals.rs`.
