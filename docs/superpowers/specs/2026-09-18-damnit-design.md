# damnit design

`damnit` is a local, git-shaped task and calendar store with a command-line client named `dam`.
Tasks and events live on your machine in a form you can stage, commit and push. Todoist, Google
Calendar and any other service are remotes: mirrors that `dam` pushes to and pulls from through a
helper program, the way git reaches Mercurial through `git-remote-hg`.

This document is the design for version one of the `dam` tool. The two clients that follow it,
`damnit.nvim` and `herdr-damnit`, get their own specifications once this ships.

## Why it exists

Task managers have no local copy, no staging, no history and no schema. Every read is a network
call, every write is immediate and irreversible, and nothing stops a task from holding two labels
that were meant to be mutually exclusive. Agents suffer most: they parse rendered text and write it
back, they cannot tell their own change from a human's, a five-call batch can fail halfway, and
nothing tells them what changed since they last looked.

Every one of those is a problem git solved for source code. `dam` applies the same shape to tasks.

## Principles

1. `dam` is the system of record. A remote is a projection of it. Fields a remote cannot hold stay
   in `dam`.
2. Reads run against local storage. A remote's `stale` setting can opt a read into a pull first;
   with it unset, no read touches the network.
3. Nothing leaves the machine until `dam push`. Nothing arrives until `dam pull`.
4. `dam` never deletes on your behalf. A removal upstream becomes a line in `dam status`. Only
   `dam rm` deletes.
5. `dam` enforces the model. A refusal says why. `--force` overrides it, and `--interactive` asks
   what to do instead.
6. Git's words where git has one: `add`, `commit`, `push`, `pull`, `status`, `diff`, `log`, `show`,
   `mv`, `rm`, `reset`, `remote`.

## Naming

| Thing | Name |
|---|---|
| Repository and crate | `damnit` |
| Binary | `dam` |
| Remote helper for Todoist | `dam-remote-todoist`, ships in this repository |
| Remote helper for Google Calendar | `dam-remote-gcal`, ships in this repository, second |
| Neovim client | `damnit.nvim`, separate repository, separate spec |
| herdr client | `herdr-damnit`, separate repository, separate spec |

`dammit` is reserved on crates.io alongside `damnit` and points at the same package.

## Data model

Two kinds of object share one base. Everything below lives in SQLite under
`~/.local/share/dam/`.

### Shared base

| Field | Type | Notes |
|---|---|---|
| `oid` | string | `dam`'s own identifier, assigned at creation, shown abbreviated. See below. |
| `subject` | string | One line. Git's word; no remote's word is privileged. |
| `body` | string | Free text. |
| `path` | string | Where the object sits in the tree. See below. |
| `labels` | set of strings | Free labels plus category values. See categories. |
| `depends` | list of `oid` | Objects that must finish first. Cycles are refused at write time. |
| `reminders` | list | Offsets or absolute times. One model for both kinds. |
| `recurrence` | rule, optional | See recurrence. |

An `oid` is `dam`'s, assigned the moment an object is created and shown as a 7-character prefix the
way git shows an object name. A remote's id for the same object is the helper's business and lives in
a mapping table that `dam` never displays.

### Task

| Field | Type | Notes |
|---|---|---|
| `done` | bool | |
| `priority` | 1 to 4 | 1 is highest. |
| `due` | date or datetime with timezone, optional | |
| `deadline` | date, optional | Distinct from `due`, as Todoist distinguishes them. |
| `event` | `oid`, optional | The event this task is attached to. |

### Event

Events have no `done`. They pass. A past event is shown as such and needs no action.

| Field | Type | Notes |
|---|---|---|
| `start`, `end` | date or datetime with timezone | A `date` means all day. `end` is exclusive. |
| `timezone` | IANA name | Required when `recurrence` is set. |
| `location` | string | |
| `attendees` | list | Email plus response: accepted, declined, tentative, needs action. |
| `status` | confirmed, tentative, cancelled | A removed occurrence of a series arrives cancelled. |
| `transparency` | busy or free | Free does not block time. |
| `visibility` | default, public, private, confidential | |
| `event_type` | default, focus time, out of office, working location, birthday | |
| `color` | string | |
| `organizer` | email and name | Read from the remote; written only on import. |
| `conference` | structured | Meeting link and provider. |
| `attachments` | list | URL, title, mime type. |

Every field in this table maps to one on Google Calendar's event resource, so an event managed in
`dam` and one managed in Google Calendar carry the same information.

### The tree

`path` is a slash-separated location, the way a file has a directory. A project is a task with
children. A section is a task with children. Depth has no meaning to `dam`; the Todoist helper maps
depth 0 to projects, depth 1 to sections and deeper levels to tasks and subtasks, and refuses a
shape Todoist cannot hold.

Because a project is a task, it has a priority, a due date and a `done` flag. Todoist has none of
those for projects; the helper translates `done` on a project to archive.

### Categories

`dam` ships no categories. Users declare them in config:

```toml
[category.effort]
values = ["light", "admin", "deep"]
exclusive = true

[category.context]
values = ["home", "office", "errand"]
exclusive = false
```

A category is a named set of label values. `exclusive = true` means an object may hold at most one
of that category's values. `exclusive = false` means any number. A value belongs to exactly one
category. Labels that belong to no category are free and always allowed, so `deep` with `errand`
with `waiting` is fine, and `deep` with `light` is refused. Helpers write every label upstream as a
plain label; the rule lives in `dam` only.

### Dependencies and completion

`dam done X` is refused while any `oid` in `depends` is open, or while any child of `X` is open.
The refusal lists the blockers. `--force` completes it anyway. `--force --interactive` asks first:

- for open children: move them up one level, into a new task at `X`'s level that you name, or keep
  them under `X`
- for open dependencies: drop the dependency or keep it

`done.interactive = true` in config makes every `--force` ask, so the flag is the opt-in and the
setting is the default.

### Recurrence

A rule has a frequency, an interval, optional by-day and by-month-day terms, an optional end, and one
bit Todoist made necessary: whether the next occurrence counts from the due date or from the
completion date. Todoist writes the second as `every!`.

`dam` computes the next occurrence itself when a recurring task is completed, so it works with no
remote attached. A helper may push the rule to a remote that understands it. Both sides then roll
forward, and `dam pull` reconciles the rare case where they land on different dates as an ordinary
conflict.

### Attachment between kinds

A task's `event` field names an event by `oid`. An event lists its tasks by looking that up. Many
tasks may attach to one event, an event needs none, a task needs none.

The link is `dam`-only data. A pull that rewrites an event from Google touches the event's own
fields and nothing else, so attached tasks survive. A cancelled event keeps its tasks; `dam status`
reports them and leaves the decision to you.

## Layers

Four layers, all in SQLite, modeled on git:

| Layer | Git equivalent | Written by |
|---|---|---|
| Working | working tree | `new`, `done`, `edit`, `mv`, `rm` |
| Stage | index | `add`, `reset` |
| Commits | local history | `commit` |
| Remote state | remote-tracking refs | `pull`, `push` |

No task files sit on disk. `dam edit -e` opens a temporary file the way `git commit` opens a message
file, and parses it back on save.

## Commands

### Writing

```
dam new "buy oat milk" [--path inbox/] [--due tomorrow] [-p 1] [--label errand]
dam new --event "Dentist" --start 2026-09-25T14:00 --end 2026-09-25T15:00
dam done <oid> [--force] [--interactive]
dam edit <oid> --subject "..." --due ... -p ... --label ... --attach <event-oid>
dam edit <oid> -e
dam mv <oid> <path>
dam rm <oid>
```

Every flag on `edit` is structured and cannot be malformed. `-e` opens the object in `$EDITOR` as a
commented template; on save `dam` parses it, and a parse failure names the problem and reopens with
your text intact. Nothing reaches the working layer until it parses.

### Staging and committing

```
dam add <oid>...            stage
dam add -A                  stage everything
dam reset [<oid>...]        unstage one or all
dam status                  working versus stage versus last commit, and remote notices
dam diff [--staged]         working versus stage, or stage versus last commit
dam commit -m "triage"      record the stage as a local commit
dam log                     history
dam show <oid|commit>       one object, or one commit
```

### Remotes

```
dam remote add todoist todoist::
dam remote add gcal gcal::
dam push [<remote>]         send unpushed commits
dam pull [<remote>]         fetch and merge
dam resolve <oid> --ours | --theirs
```

`push` with no remote sends to every remote. Each helper declares in its capabilities which kinds
and fields it accepts, and `dam` sends each object to every remote that accepts it. A remote may be
narrowed to a `path` in config.

`pull` is per remote and never implies another. Each remote has its own `stale` setting.

### Reading

```
dam ls [<query>] [--json]
dam ls today                a saved filter from config
dam show <oid> [--json]
dam status [--json]
```

The query is a string in one grammar for both kinds:

```
due:today | overdue
path:webdavis/dotfiles/ & effort:deep & !done
start:this-week & transparency:busy
attached:<event-oid>
```

Saved filters live in config and run by name:

```toml
[filter.today]
query = "due:today | overdue"
```

Every read command takes `--json` and prints one document per line. This is the interface the
clients use.

## Remotes and helpers

`dam` core contains no Todoist code and no Google code. A remote is reached through a helper found
on `PATH` by name, exactly as git finds `git-remote-https`:

| Git | `dam` |
|---|---|
| `hg::<address>` picks `git-remote-hg` | `todoist::` picks `dam-remote-todoist` |
| Helper on `PATH`, any language | Same |
| Fixed protocol on stdin and stdout | JSON lines on stdin and stdout |
| `capabilities` handshake | Same, with a protocol version |
| Credential helpers | `dam` resolves the token and hands it to the helper in its environment |

A missing helper is reported by name.

### Protocol

One JSON object per line in each direction. The first exchange is always `capabilities`:

```json
{"cmd": "capabilities"}
{"protocol": 1, "kinds": ["task"], "fields": ["subject", "body", "path", "labels", "priority",
 "due", "deadline", "done", "recurrence"], "incremental": true}
```

`kinds` and `fields` are what the helper will accept on push and return on pull. A helper never
receives a field it did not declare, and `dam` never lets a pulled object overwrite a field the
helper did not declare. That rule is what keeps `dam`-only data safe.

```json
{"cmd": "pull", "since": "<opaque sync token or null>"}
{"objects": [...], "removed": ["<remote-id>", ...], "sync": "<opaque token>"}

{"cmd": "push", "mutations": [{"op": "create|update|delete", "oid": "...", "fields": {...}}, ...]}
{"results": [{"oid": "...", "ok": true, "remote_id": "..."}, {"oid": "...", "ok": false, "why": "..."}]}
```

Results are per mutation. Successes leave the unpushed set; failures stay with their reason and
appear in `status`. A helper that supports incremental sync returns a token; one that does not
returns null and `dam` diffs the full set itself.

The protocol version only grows. A helper written against version 1 keeps working against every
later `dam`.

### Credentials

Each remote names a `token_command` in config. `dam` runs it, takes the first line of standard
output, and passes the value to the helper as `DAM_REMOTE_TOKEN` in its environment. The token
never appears in an argument, a log line or a status message.

### Helpers in this repository

`dam-remote-todoist` ships in version one. `dam-remote-gcal` ships second. Both are binaries in the
`damnit` package, so `cargo install damnit` installs `dam` and both helpers together.

A native remote, `dam-remote-https` against a server that speaks this protocol, is a later
helper and out of scope here.

## Sync rules

1. `pull` never overwrites uncommitted local work. If the remote changed an object you have in the
   working layer or the stage, `pull` stops on that object and reports it. Commit or reset, then
   pull again.
2. A conflict is an object both sides changed since the last common state. `pull` marks it; `dam
   status` lists it; `dam resolve <oid> --ours | --theirs` settles it. Nothing is merged
   automatically.
3. A removal upstream is a notice, never a local deletion.
4. An event cancelled upstream keeps its attached tasks and is reported.
5. A helper only writes the fields it declared.
6. Recurring objects that both sides rolled forward to different dates are conflicts like any other.

## Configuration

`~/.config/dam/config.toml`:

```toml
[done]
interactive = true

[remote.todoist]
url = "todoist::"
token_command = ["security", "find-generic-password", "-w", "-s", "Todoist API Token"]
stale = "15m"

[remote.gcal]
url = "gcal::"
token_command = ["gog", "auth", "token"]
stale = "5m"
path = "calendar/"

[category.effort]
values = ["light", "admin", "deep"]
exclusive = true

[filter.today]
query = "due:today | overdue"
```

`stale` makes a read command pull that remote first when the last pull is older than the value.
Absent, reads never pull.

Storage is `~/.local/share/dam/dam.db`, one SQLite file in WAL mode, mode 0600.

## Performance

Reads are local SQLite queries. The targets, measured on a store of ten thousand objects:

- `dam ls <query> --json`: under 20 ms wall clock, process start included
- `dam status`: under 30 ms
- `dam pull` against Todoist with a sync token: bounded by the network, one request

The clients spawn `dam ... --json` per render and read the result. No daemon and no socket in
version one; the process start cost is measured before that is reconsidered.

## Errors

A refusal names the rule and the objects involved, then stops. A missing helper is named. A failed
push lists each failed mutation with the helper's reason. A parse failure in `edit -e` names the line
and reopens. No error message contains a token.

## Testing

- Every crate has unit tests; every test finishes within one second.
- Helpers are tested against a loopback double of the upstream API, never the live service.
- The protocol is pinned by golden fixtures: one file per command and response, shared by `dam`
  and every helper in the repository, so a change that moves the bytes fails a test on both sides.
- The query grammar has a table-driven parser test.
- Enforcement rules (categories, dependencies, children, cycles) each have a red test before the
  code that satisfies it.

## Crate layout

```
damnit/
  Cargo.toml                 workspace
  crates/
    dam/                     the binary: argument parsing, output, editor round trip
    dam-core/                model, storage, staging, commits, query, enforcement, sync
    dam-protocol/            wire types and the golden fixtures
    dam-remote-todoist/      binary
    dam-remote-gcal/         binary, second
  docs/superpowers/specs/
```

Rust follows the clean-code standard for Rust. Files target 300 lines and never exceed 500, tests
included. No crate depends on anything outside this workspace except published crates.

## Out of scope for version one

- `damnit.nvim` and `herdr-damnit`, each its own spec
- Google Calendar push notifications; `stale` covers the gap
- `dam-remote-https` and any server
- A capacity or scheduling model over free windows; the data for it is present, the logic is not
- Comments and attachments on tasks
- Assignees and shared projects

## Decisions recorded

Made in the design conversation on 2026-09-18:

- Name: `damnit`, binary `dam`, helpers `dam-remote-<name>`.
- Full local mirror, not a staging area alone.
- Four layers in SQLite; no task files on disk; `add` means stage; `new` creates; `commit` exists.
- Remote helpers on `PATH`, JSON lines, versioned capabilities, git's exact shape.
- A project is a task with children. Projects have priority, due and done.
- Categories are user-defined, exclusive or not, values unique across categories, free labels
  always allowed.
- `depends` with `--force` and `--interactive`; children enforced the same way; `done.interactive`
  in config.
- Recurrence computed by `dam`, optionally shared with a remote, reconciled by `pull`; the
  from-completion bit is modeled.
- Events are a second kind on the shared base, with every Google Calendar field, no `done`.
- Tasks attach to events by `oid`; the link is `dam`-only and survives any pull.
- Pull is per remote; several remotes push at once; each helper declares what it accepts.
- `dam` never deletes on your behalf.
- Query string with saved filters, over flags.
- `subject` and `body` over `title` and `description`.
