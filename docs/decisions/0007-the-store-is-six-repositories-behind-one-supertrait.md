# The store is six repositories, not one interface

## Context

The application originally reached persistence through one trait of about thirty-five methods,
covering objects, the stage, commits, remote tracking, conflicts and notices. Every use case
depended on all of it, every test double implemented all of it, and a use case's signature said
nothing about which records it touched.

## Decision

Six ports, one per record family: objects, stage, commits, remote tracking, conflicts and notices.
A use case takes the ports it uses and no others. `Store` is the supertrait an adapter implements
and `Repositories` is the bundle the composition root hands to a use case that needs several.
`Transactional` is separate again, because a unit of work is not a record family.

## Consequence

A use case's signature now names what it reads and writes. A test double for one family is a few
lines rather than a stub of thirty-five methods, and one contract suite runs against both the SQLite
store and the in-memory double, so the double cannot drift from the real thing.

What is not caught: nothing checks that the six ports, the `Store` supertrait and what `Repositories`
builds stay in agreement. The compiler catches a missing implementation, not a port added to `Store`
and forgotten in the bundle.

Where the code says it: `crates/dam-application/src/ports.rs` and
`crates/dam-application/src/testing/contract.rs`.
