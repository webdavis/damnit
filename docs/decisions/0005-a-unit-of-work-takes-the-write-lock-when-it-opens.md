# A unit of work takes the write lock when it opens

## Context

The store is SQLite in WAL mode, and the design puts several `dam` processes behind one client
render, so concurrent writers are the expected case rather than a hypothetical.

A transaction that begins deferred takes no lock until its first write. Several write paths read
before they write: recording a commit reads the next sequence number, mapping a remote id reads the
existing mapping, staging reads the change already staged. In WAL mode, a deferred transaction that
read first and then tries to write after another connection has committed answers `SQLITE_BUSY`
immediately, and SQLite does not consult the busy handler for that case because it cannot safely
retry. The connection's five second busy timeout therefore did not cover any of those paths.

## Decision

The outermost unit of work opens with `BEGIN IMMEDIATE`, which takes the write lock before the
first statement. A nested one opens a savepoint, so an inner unit of work behaves the same whether
or not an outer one is already open.

## Consequence

A second writer is locked out for the whole unit of work rather than only from its first write, and
the busy timeout covers the wait. A `SQLITE_BUSY` that does surface is its own outcome, so a caller
can tell contention from a permanent failure.

The cost is that a unit of work holds the write lock for its full duration, including its reads.
The units are short and the alternative is losing a write.

Where the code says it: `in_savepoint` in `crates/dam-adapters/src/sqlite/mod.rs`.
