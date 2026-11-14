# A remote that removes only empty containers decides a same-name removal

## Context

`dam` knows what holds an object only by its path. Two objects with the same name in the same place
own the same folder. When one of them is removed and the other stays, `dam` cannot tell which of
them holds a child upstream, so under decision 0011 the delete waits forever on a child that may
sit under the other one. Todoist deletes everything under a deleted task, section or project, so
sending the delete without knowing is not safe either.

## Decision

A helper may declare `removes_only_empty_containers` in its capabilities. A helper that leaves it
out declares false.

A delete is held only because of a namesake when it would be stuck, and another object that the
remote holds and that this push does not remove owns the same folder as the deleted object. `dam`
reads what the remote holds from its remote snapshot, including what this push placed earlier.

- For a remote that declares `removes_only_empty_containers`, `dam` sends such a delete after
  everything it waits on has been sent, and the helper's own check decides. It does so only when
  every object still under the folder upstream sits where the person keeps it, and this push
  placed it, or the object directly in the folder that holds it, and the remote accepted that.
  Otherwise the delete stays held, because the helper's check cannot see everything a container
  holds.
- For any other remote, the delete stays held. The reason names the namesake and asks the person to
  rename it, then push. A rename moves the namesake's children into a folder of its own, so the
  next push sends the delete.
- A delete stuck for any other reason stays held, whatever the remote declares.

When a push removes the owner of a folder and exactly one other object owns that folder after the
push, every object the remote holds directly in that folder and that stays there has its place
sent. An object deeper in the folder is not sent, since its own folder keeps its owner. When the
remote already holds the one owner that stays, the mutation names it in `container_remote_id`, so
the helper files the object under that container and not under the namesake being removed. When
two or more owners stay, `dam` cannot say which one holds the object, so it sends nothing and the
delete stays held.

The Todoist helper declares it. Before it sends a delete, it checks its tree by id and refuses that
one mutation when Todoist still shows a task under the task, a task in the section, or a task,
section or project in the project. The tree comes from the read at the start of the call and
follows every create, move and delete the call makes, so an inner delete or a move out earlier in
the same call empties the container first. A task whose mutation names a container goes under that
container. Otherwise a task filed under a container that this push created uses the id Todoist
issued for it, never a lookup by name.

## Consequence

The rule stays in the application and reads a declared capability, so it is the same for every
remote. A remote that cannot check gives the person a hold with a way out.

The Todoist check sees only what the helper's one read returns: projects, sections and tasks. The
helper makes no separate call for completed tasks or archived projects, so a completed subtask
under an otherwise empty container is not seen and is deleted with it, as before.

Where the code says it: `crates/dam-application/src/use_cases/push/refiles.rs`,
`crates/dam-application/src/use_cases/push/removals/blocker.rs`,
`crates/dam-application/src/use_cases/push/removals/release.rs`,
`crates/dam-remote-todoist/src/map/holdings.rs`,
`crates/dam-remote-todoist/src/push/plan/removal.rs` and
`crates/dam-remote-todoist/src/push/plan/update.rs`.
