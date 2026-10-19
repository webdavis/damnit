# The dam helper protocol

A remote is reached through a helper executable named `dam-remote-<helper>` on `PATH`. `dam`
writes requests to the helper's standard input and reads responses from its standard output, one
JSON object per line in each direction. `dam` keeps the tail of the helper's standard error and
quotes it when the helper fails to answer.

The golden fixtures in `crates/dam-protocol/fixtures` pin each message shape below.

## Invocation

`dam` runs the helper as `dam-remote-<helper> <remote> <address>`. `<remote>` is the remote's
name. `<address>` is the text after `::` in its url, empty when the url ends at `::`. Both are
always passed, and neither is ever a secret. A helper reads its credentials under the remote name.

## Version and compatibility

`PROTOCOL_VERSION` is the version `dam` speaks. A helper declares its own in the `protocol` field of
its capabilities answer.

1. **Unknown fields are ignored.** A helper may add a field to any response, and an older `dam`
   reads the rest of it. A field keeps its meaning in every later version.
2. **Unknown message kinds are refused.** A response lacking its answer's keys is a protocol error,
   not an empty answer. A helper answers a request for a command it does not know with an error
   object.
3. **A helper above `PROTOCOL_VERSION` is refused by name.** `dam` drives every version from 1 up
   to its own and refuses anything above it, naming both numbers. A helper declaring version 0 is
   refused.

`dam` refuses a line longer than `MAX_LINE` bytes, its newline included. A conversation has no
length limit.

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

`dam` reads each response as the answer to the request it sent. Any response carrying `error` is
a failure. Any other response must carry one of the answer's required keys.

| Response | Requires one of | Fields |
|---|---|---|
| capabilities | `protocol` | `protocol`, `kinds`, `fields`, `credentials`, `incremental` |
| pull | `objects`, `rejected`, `removed`, `cancelled`, `sync` | the same five |
| push | `results` | `results`, one entry per mutation |
| error | `error` | `error`, the reason as text |

`removed` lists remote ids the remote no longer has. `dam` stops tracking each one.

`cancelled` lists remote ids of events the remote cancelled without restating the object. `dam`
moves the event it tracks under each id to `cancelled` and keeps tracking it. An id it does not
track, or one that names a task, changes nothing. A helper that sends `cancelled` declares `status`
in `fields`; otherwise each id changes nothing.

`rejected` lists objects the helper could not read, each as `{"remote_id", "why"}`. `dam` lands the
rest of the pull and shows each entry in `dam status` as a notice naming the id and the reason. An
object `dam` itself cannot read is reported the same way.

One push result:

| Field | Meaning |
|---|---|
| `oid` | the mutation this answers |
| `ok` | whether the remote did it |
| `remote_id` | the remote's identity for the object, after a create |
| `why` | the reason, when `ok` is false |

## Capabilities

`kinds` is what the helper accepts on push, and `fields` is what it accepts on push and returns on
pull. A helper that declares no kinds is read-only. `dam` sends it no mutation, and a push marks
its commits pushed with nothing refused or retried. A helper never receives a field it did not
declare, and a pulled object never overwrites a field the helper did not declare.

`credentials` names what the helper needs. `dam` resolves each one and passes it in the helper's
environment as `DAM_<REMOTE>_<NAME>`, uppercased, with every non-alphanumeric character turned into
an underscore. No value reaches an argument, a log line or a status message.

`incremental` says whether the helper honors `since`. A helper that does returns a token in `sync`.
One that does not returns null, and `dam` diffs the full set itself.

## Objects

`kind` is `task` or `event`. A task carries a `task` table and an event an `event` table.

A task's `completed_at` is when it was completed, as RFC 3339 in UTC, and its `event` is the oid of
the event it is attached to. Both are `dam`'s own data.

An event's `start` and `end` are a date or a zoned time. Its words take these values:

| Field | Values |
|---|---|
| `status` | `confirmed`, `tentative`, `cancelled` |
| `transparency` | `busy`, `free` |
| `visibility` | `default`, `public`, `private`, `confidential` |
| `event_type` | `default`, `focus_time`, `out_of_office`, `working_location`, `birthday` |
| attendee `response` | `needs_action`, `accepted`, `declined`, `tentative` |

An attendee carries `"self": true` when it is the calendar the event was read from. The key is
written only when true, and its absence reads as false.

## Delivery

Delivery is at least once. `dam` sends an unanswered mutation again with the same
`idempotency_key`, a value derived from the commit and the object. A helper passes the key to a
remote that deduplicates by key, so the work happens once. A helper that turns one mutation into
several remote commands numbers them in the key's last character, which `dam` leaves free. Against
a remote with no such key, a resend can duplicate.
