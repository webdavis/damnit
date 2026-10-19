# Store rows are wire JSON

## Context

One type, `WireObject` in `dam-protocol`, is three formats at once: the object a helper sends and
receives, the JSON row the SQLite store keeps for each object, and the object `--json` prints.
A change to it changes all three, and no single test said so.

## Decision

The three formats stay one type. Golden tests pin each of them instead of a split into three types.

## Consequence

A change to `WireObject` fails a golden in each format it touches. The stored-row goldens hold the
exact row of a fully populated task and a fully populated event, and every event enum word. The CLI
goldens hold every `--json` document and the human lines of the reads. A deliberate change updates
the golden files with `GOLDEN_BLESS=1` in the same commit, so the diff shows what readers will see.

The stored row never carries `remote_id`; remote ids live in the store's `remote_ids` table.

Where the code says it: `crates/dam-adapters/src/sqlite/codec.rs`, `crates/dam-cli/src/output.rs`,
`crates/dam-adapters/tests/stored_rows.rs` and `crates/dam-cli/tests/golden/`.
