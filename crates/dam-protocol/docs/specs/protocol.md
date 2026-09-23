# The dam helper protocol

A remote is reached through a helper executable `dam` finds on `PATH` by the name
`dam-remote-<remote>`, the way git finds `git-remote-https`. `dam` writes requests to the helper's
standard input and reads responses from its standard output, one JSON object per line in each
direction. The helper's standard error is the helper's own diagnostics: `dam` keeps the tail of it
and quotes it when the helper fails to answer.

This document is the contract a third-party helper is written against. Everything it states is
pinned by golden fixtures in this crate.

## Version and compatibility

`PROTOCOL_VERSION` is the version `dam` speaks. A helper declares its own in the `protocol` field of
its capabilities answer.

The version only grows, and these three rules are what "grows" means:

1. **Unknown fields are ignored.** A helper may add a field to any response and an older `dam`
   reads the rest of it. Add fields, never repurpose one.
2. **Unknown message kinds are refused.** A response matching no known shape is a protocol error,
   not an empty answer. A request naming a command the helper does not know is answered with an
   error object.
3. **A helper above `PROTOCOL_VERSION` is refused by name.** `dam` drives every version up to its
   own and refuses anything above it, naming both numbers, rather than driving a newer helper with
   older semantics. A helper written against version 1 keeps working against every later `dam`.

A line longer than `MAX_LINE` bytes, its newline included, is refused. The budget is per line, so
the length of a conversation is not bounded.

## Requests

| Request | Fields |
|---|---|
| `{"cmd": "capabilities"}` | none |
| `{"cmd": "pull", "since": <string or null>}` | `since` is the opaque token from a previous pull |
| `{"cmd": "push", "mutations": [...]}` | see the mutation table |

One mutation:

| Field | Meaning |
|---|---|
| `op` | `create`, `update` or `delete` |
| `oid` | the object's identity in `dam`, forty hex characters |
| `idempotency_key` | stable across every resend of this same mutation |
| `remote_id` | the remote's own identity for it, absent on a create |
| `object` | the object itself, absent on a delete |
| `fields` | the field names this mutation changes |

## Responses

A response is recognized by the keys it carries, not by the order the variants are declared.

| Response | Recognized by | Fields |
|---|---|---|
| capabilities | `protocol` | `protocol`, `kinds`, `fields`, `credentials`, `incremental` |
| pull | `objects`, `removed`, `cancelled` or `sync` | `objects`, `removed`, `cancelled`, `sync` |
| push | `results` | `results`, one entry per mutation |
| error | `error` | `error`, the reason as text |

`cancelled` lists remote ids the remote cancelled, for a remote that reports a cancellation without
restating the object, as Google Calendar does for a deleted event. `dam` moves the event it tracks
under each id to `cancelled` and keeps tracking it; an id it does not track, or one that names a
task, changes nothing. `removed` stays what it was: the remote no longer has the object, and `dam`
stops tracking it.

`error` is looked for first, so a response carrying both `error` and a shape's own keys is read as a
failure.

One push result:

| Field | Meaning |
|---|---|
| `oid` | the mutation this answers |
| `ok` | whether the remote did it |
| `remote_id` | the remote's identity for the object, after a create |
| `why` | the reason, when `ok` is false |

## Capabilities

`kinds` and `fields` are what the helper accepts on push and returns on pull. A helper never
receives a field it did not declare, and a pulled object never overwrites a field the helper did not
declare. That rule is what keeps `dam`-only data safe.

`credentials` names what the helper needs. `dam` resolves each one and passes it in the helper's
environment as `DAM_<REMOTE>_<NAME>`, uppercased with every non-alphanumeric character folded to an
underscore. No value reaches an argument, a log line or a status message.

`incremental` says whether the helper honors `since`. A helper that does returns a token in `sync`;
one that does not returns null and `dam` diffs the full set itself.

## Delivery

Delivery is at least once. `dam` cannot tell a request that never arrived from an answer that never
came back, so it sends the mutation again, and the same mutation always carries the same
`idempotency_key`: a value derived from the commit and the object, never minted afresh for a
resend. A helper passes it to a remote that deduplicates by key, so the work happens once however
often it is sent. A helper that turns one mutation into several remote commands numbers them in the
key's last character, which `dam` leaves free for that. Against a remote with no such key, delivery
is at least once and a resend can duplicate.
