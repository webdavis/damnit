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
| `reminders` | list | Minutes before the due time. Version one models only that offset. |
| `recurrence` | rule, optional | See recurrence. |

An `oid` is `dam`'s, assigned the moment an object is created and shown as a 7-character prefix the
way git shows an object name. A remote's id for the same object is the helper's business and lives in
a mapping table that `dam` never displays.

### Task

| Field | Type | Notes |
|---|---|---|
| `done` | bool | |
| `completed_at` | timestamp, optional | When the task was completed, in UTC. |
| `priority` | 1 to 4 | 1 is highest. |
| `due` | date or datetime with timezone, optional | |
| `deadline` | date, optional | Distinct from `due`, as Todoist distinguishes them. |
| `event` | `oid`, optional | The event this task is attached to. |

`completed_at` is `dam`'s own record of when a task was finished: `dam done` sets it to the moment it
ran, `dam edit --undone` clears it, and it goes with `done` whenever that field moves, so an open
task never carries one. It is written as RFC 3339 in UTC wherever a document carries it. Todoist
holds a completion time of its own, on its completed-tasks endpoint rather than on the sync item
`dam` reads, and its complete command takes only an id; the Todoist helper therefore declares neither
the field nor a value for it, and a task Todoist completed arrives done with `completed_at` unset.

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

`--children up|keep|into:<name>` and `--depends drop|keep` give those same answers on the command
line, one word per answer the prompt offers, with the group's name carried inline so `--children`
needs no second value. Either flag turns the prompt off for that run whatever `done.interactive`
says, which is what lets a client complete a blocked parent with no terminal at all, and the flag
that is absent takes `keep`, the answer that disturbs least. `--force` on its own still asks when
`done.interactive` is set. A word outside the set is a usage error, either flag without `--force`
is refused, and so is either flag beside an explicit `--interactive`: the setting is a default to
override, a flag the operator typed is not.

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
dam done <oid> --force --children up|keep|into:<name> --depends drop|keep
dam edit <oid> --subject "..." --due ... -p ... --label ... --attach <event-oid>
dam edit <oid> --undone
dam edit <oid> -e
dam mv <oid> <path>
dam rm <oid>
```

`dam edit <oid> --undone` reopens a completed task. It is a flag on `edit` rather than a verb of
its own, because `done` is a field like any other and the flag sits beside `--no-due`,
`--no-deadline`, `--no-recurrence` and `--detach`, which already pair a clearing flag with the flag
that sets the field. Reopening records one ordinary working change, op `update` with `done` among
its fields, so it stages, commits and pushes the way every other edit does; the Todoist helper
sends it as the Sync API's `item_uncomplete`. A task that is not completed is refused, naming it.

Every flag that takes a date, `--due`, `--deadline`, `--start` and `--end`, reads one grammar:
`today`, `tomorrow`, a weekday by short or full name, `next` before a weekday, `in <n>` days, weeks
or months, `YYYY-MM-DD`, and `YYYY-MM-DDTHH:MM` or a full zoned timestamp for a flag that holds a
time. Words are case-insensitive. A weekday means its next occurrence strictly after today, so a
weekday named on its own day is a week out; `next mon` means that same day rather than the Monday of
the following calendar week, because an operator has no way to say which of those two they meant and
the sooner date is the recoverable mistake. A count of months lands on the same day of a later
month, clamped to that month's last day. A deadline is a whole day, so a text carrying a time keeps
its date and drops the time. A word outside the grammar is the command line being wrong rather than
a rule being broken: exit 2, kind `usage`, and the message names every accepted form. A deadline
already in the past is a date like any other.

Every flag on `edit` is structured and cannot be malformed. `-e` opens the object in `$EDITOR` as a
commented template; on save `dam` parses it, and a parse failure names the problem and reopens with
your text intact. Nothing reaches the working layer until it parses.

### Staging and committing

```
dam add <oid>...              stage
dam add -A                    stage everything
dam reset [<oid>...]          unstage one or all
dam restore <oid>...          back to the last commit, working and stage together
dam status [--full]           working versus stage versus last commit, and remote notices
dam diff [--staged] [--full]  working versus stage, or stage versus last commit
dam commit -m "triage"        record the stage as a local commit
dam log                       history
dam show <oid|commit>         one object, or one commit
```

### Remotes

```
dam remote add todoist todoist::
dam remote add gcal gcal::
dam push [<remote>]         send unpushed commits
dam pull [<remote>]         fetch and merge
dam resolve <oid> --ours | --theirs
```

`dam restore <oid>...` sets each named object's working copy back to its last commit and drops
whatever the stage held for it, so afterwards the object equals the commit and nothing about it is
staged. The name is git's own since 2.23, and `dam reset` already means unstage here too. An object
with no commit behind it is refused, naming it, because there is nothing to go back to; `dam rm`
removes such an object instead. An oid neither layer knows, which is what a committed delete leaves
behind, is refused as no such object rather than sent to `dam rm`. An object that already matches
its commit is a no-op that says so. Every named oid is read before any is written, so one refusal
leaves the others as they were, and a repeated oid answers once. Under `--json` the document lists
the restored oids and the unchanged ones.

An oid prefix is resolved against the working layer, here as in every verb, so an object `dam rm`
took out of it is named by its full oid and the refusal on a prefix says which layer it read.

`dam remote add --json` answers with the remote's name and url and a `warnings` list, empty in the
ordinary case. It carries one sentence when the config file had been readable by someone other than
its owner, which `dam` tightens as it writes: the file is the documented home of a literal
credential, so a client driving `remote add` is told rather than left to notice.

`push` with no remote sends to every remote. Each helper declares in its capabilities which kinds
and fields it accepts, and `dam` sends each object to every remote that accepts it. A remote may be
narrowed to a `path` in config.

`pull` is per remote and never implies another. Each remote has its own `stale` setting.

`dam remote list` reports, per remote, when `dam` last pulled it and last pushed to it. A verb
records its time once the run reached the remote and came back with an answer, the way `pull`
already records its own: a mutation the remote refused is a notice against that object rather than a
failed run, so a push whose mutations were partly refused still records the time. A push with
nothing to send records it too, on the strength of the capabilities exchange that opened the
connection, so a remote that is entirely up to date does not read as never pushed. `--json` carries
`last_pull` and `last_push` as RFC 3339 timestamps, null for a verb that has not reached that remote
yet; the human line says `pulled 4m ago` and `pushed 2h ago`, or `never pulled` and `never pushed`.
Both are kept in the store beside the sync token, one row per remote.

### Reading

```
dam ls [<query>] [--json|--toon] [--no-pull]
dam ls today                a saved filter from config
dam show <oid> [--json|--toon] [--no-pull]
dam status [--json|--toon] [--full]
dam category list [--json|--toon]
dam filter list [--json|--toon]
```

`--no-pull` answers from the local store: the reads that would otherwise pull a stale remote first,
`ls` and `show`, skip that pass and spawn no helper. The documents are unchanged, and nothing in
them reports staleness; a client that renders inside a frame budget uses the flag to make the read
local, and reads `remote list` when it wants to say how fresh the answer is. Neither the `stale`
setting nor anything else in config changes.

`ls` and `show` are the only verbs it applies to, so every other verb refuses it as a command line
that was wrong, exit 2, naming the flag. `dam pull --no-pull` is refused like the rest rather than
read as a contradiction.

`--json` is the answer as JSON. `--toon` is the same answer in TOON (Token-Oriented Object
Notation), a compact form that costs a language model fewer tokens when it reads a list; the
helper protocol and the store stay JSON, because no model reads those.

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

`dam category list` and `dam filter list` print what config declares, so a client groups a label
picker by category, pre-refuses an exclusive clash and offers the saved filters as views without
owning a second parser for a file `dam` owns. Each answers in the shape `remote list` uses, one
top-level key holding an array of objects with fixed keys:

```json
{"categories": [{"name": "effort", "values": ["light", "admin", "deep"], "exclusive": true}]}
{"filters": [{"name": "today", "query": "due:today | overdue"}]}
```

Both list in alphabetical order by name, which is the order `dam`'s config reader hands them over
rather than the order the file declares them; a client that wants another sorts what it is given.

Nothing declared is an empty array rather than a failure, so a client renders an empty picker. The
human line is the name, then `exclusive` or `any` and the values for a category, and the query for
a filter. Neither verb reads the store or a remote, so both refuse `--no-pull` like every other
verb that never pulls, and neither opens a store at all: a store that will not open fails the verbs
that read one and never a listing of what config declares.

Every read command takes `--json` and prints one JSON document on stdout. This is the interface
the clients use.

#### The change document

`dam status`, `dam diff`, `dam log`, `dam add` and `dam commit` answer in change documents, one per
change. The default carries what a client renders and no object:

```json
{"oid": "98d878...", "op": "update", "fields": ["subject", "due"], "kind": "task",
 "subject": "buy oat milk", "path": "work/", "labels": ["errand"], "done": false,
 "completed_at": null, "priority": 1, "due": "2026-09-25"}
```

`fields` names what the change touches, so a client reads dam's own answer rather than diffing the
before and after itself. An update names the fields that moved. A create names the fields the new
object carries: the ones its kind always has, `subject` for a task and `subject`, `start` and `end`
for an event, then every other field holding something other than its default. A delete names
nothing, because it removes the object whole rather than any field of it. An update that changes an
object's kind names `kind` alone, whatever else differs between the two, because a task and an event
share no field list to compare.

The rest of the row is the state the change left behind: `kind`, `subject`, `path` and `labels` for
either kind, then `done`, `completed_at`, `priority` and `due` for a task or `start` and `end` for an
event. A delete has no such state, so its row describes the object it removed.

A completion never names `completed_at` in `fields`: the time moves only when `done` does, so `done`
is the one name that covers it, and a client reads the value off the row.

`dam status --full` and `dam diff --full` add `before` and `after`, each the whole object or null,
beside that row. The default exists because the clients poll `status` per render: three hundred
uncommitted creates measured 146,985 bytes with the objects embedded, and the row shape is what a
statusline count and a change list actually read.

#### The status document

`dam status` answers one document beside the change documents it carries: `staged` and `unstaged`
are change documents, `conflicts` is one row per object both sides changed, `notices` is what a pull
or a push left to read, and `unpushed` is one row per configured remote.

```json
{"remote": "todoist", "commits": 2, "oids": ["9f21c4...", "3b0e77..."]}
```

`oids` names the commits that remote has not been told about, newest first, which is the order
`dam log` prints history in, so a client pairs the two lists without sorting either. `commits` is
the length of that list, so a statusline reads one field and a client offering to open a commit
reads the other. A remote owed nothing carries an empty list and a zero. The human page lists only
the remotes that are owed something.

### Exit codes

One meaning per code, so a client can tell what happened without reading the message:

| Code | Meaning |
|---|---|
| 0 | The command did what it was asked. |
| 1 | `dam` failed: a store, config, helper, editor or io failure. |
| 2 | The command line was wrong: an unknown argument or subcommand, a flag value `dam` refuses to read, or an oid prefix that names more than one object. |
| 3 | Cancelled: the operator interrupted, or a prompt could not be answered. |
| 4 | `dam` refused by one of its own rules, and the message names the rule. |

Every rule `dam` keeps reports code 4, and nothing else does, so a client maps the code once
rather than per verb. An empty stage, a move into an object's own path, an event field asked of a
task and a question no machine format can answer are refusals like any other, and each names its
rule. Code 2 stays for the command line alone: an argument `dam` cannot read, and never a rule.

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
 "due", "deadline", "done", "recurrence"], "credentials": ["api_token"], "incremental": true}
```

`kinds` and `fields` are what the helper will accept on push and return on pull. A helper never
receives a field it did not declare, and `dam` never lets a pulled object overwrite a field the
helper did not declare. That rule is what keeps `dam`-only data safe.

```json
{"cmd": "pull", "since": "<opaque sync token or null>"}
{"objects": [...], "removed": ["<remote-id>", ...], "sync": "<opaque token>"}

{"cmd": "push", "mutations": [{"op": "create|update|delete", "oid": "...",
 "idempotency_key": "<uuid>", "remote_id": "...", "object": {...},
 "fields": ["<changed field name>", ...]}, ...]}
{"results": [{"oid": "...", "ok": true, "remote_id": "..."}, {"oid": "...", "ok": false, "why": "..."}]}
```

Results are per mutation. Successes leave the unpushed set; failures stay with their reason and
appear in `status`. A helper that supports incremental sync returns a token; one that does not
returns null and `dam` diffs the full set itself.

Delivery is at least once. `dam` cannot tell a request that never arrived from an answer that never
came back, so it sends the mutation again, and the same mutation always carries the same
`idempotency_key`: a UUID `dam` derives from the commit and the object, never minted afresh for a
resend. A helper passes it to a remote that deduplicates by key, so the work happens once however
often it is sent. `dam-remote-todoist` sends it as the `uuid` of each Sync API command, numbering a
mutation's several commands in the key's last character, which `dam` leaves free for that. Against a
remote with no such key, delivery is at least once and a resend can duplicate.

The protocol version only grows. A helper written against version 1 keeps working against every
later `dam`.

### Credentials

A helper declares the credentials it needs by name in `capabilities`. `dam-remote-todoist` declares
`api_token`; a Google helper declares `client_id`, `client_secret` and `refresh_token`. The remote's
name is already the table, so credential names carry no prefix. The
remote's config table provides each one in one of three ways, tried in this order when more than one
is set:

| Key | Meaning |
|---|---|
| `<name>` | The value itself, in the config file, with `<name>` listed in `credentials`. `dam status` warns when this is set. |
| `<name>_command` | Argv of a command whose first line of standard output is the value. |
| `<name>_env` | The name of an environment variable holding it. |

The config is parsed before any helper runs, so the parser cannot ask a helper which names it
declares. A bare `<name>` key is therefore read as a credential only when the remote's
`credentials` array lists that name; the suffixed forms say what they are in the key itself and
need no listing. Every other key in a remote table is refused by name, so a misspelled setting is
a refusal rather than a silently exported credential.

`dam` resolves each and passes it to the helper as `DAM_<REMOTE>_<NAME>` in its environment, so
`api_token` under `[remote.todoist]` arrives as `DAM_TODOIST_API_TOKEN`. A declared credential with
no source in config is a refusal that names the key. No value appears in an argument, a log line or
a status message.

### Helpers in this repository

`dam-remote-todoist` ships in version one. `dam-remote-gcal` ships second. Each helper is its own
package (`dam-remote-todoist`, `dam-remote-gcal`), separate from `damnit`, which builds only the
`dam` binary. `cargo install --git <url> damnit` and `cargo install --git <url> dam-remote-todoist`
are two separate installs.

`dam-remote-todoist` reads one environment variable of its own, `DAM_TODOIST_BASE_URL`, which
points it at a test server instead of Todoist. It accepts only `http://127.0.0.1:<port>` or
`http://localhost:<port>` and refuses anything else, so the variable cannot send the bearer token
to another host.

A native remote, `dam-remote-https` against a server that speaks this protocol, is a later
helper and out of scope here.

## Sync rules

1. `pull` never overwrites uncommitted local work. If the remote changed an object you have in the
   working layer or the stage, `pull` stops on that object and reports it. Commit or reset, then
   pull again.
2. A conflict is an object both sides changed since the last common state. `pull` marks it; `dam
   status` lists it; `dam resolve <oid> --ours | --theirs` settles it. Nothing is merged
   automatically. Either side is settled by a commit, so what is owed to the remote ends at the
   side that won, and a settled conflict is not raised again: a pull that finds the remote holding
   what it held when `dam` last looked has nothing coming in, whichever way the local copy has
   since moved.
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
api_token_command = ["keepassxc-cli", "show", "-a", "Password", "~/vault.kdbx", "Todoist :: API Token"]
# or: api_token = "..."          the value itself, warned about by dam status
# or: api_token_env = "TODOIST_API_TOKEN"
stale = "15m"

[remote.gcal]
url = "gcal::"
client_id_command = ["keepassxc-cli", "show", "-a", "UserName", "~/vault.kdbx", "Google :: dam"]
client_secret_command = ["keepassxc-cli", "show", "-a", "Password", "~/vault.kdbx", "Google :: dam"]
refresh_token_command = ["keepassxc-cli", "show", "-a", "refresh", "~/vault.kdbx", "Google :: dam"]
stale = "5m"
path = "calendar/"

[category.effort]
values = ["light", "admin", "deep"]
exclusive = true

[filter.today]
query = "due:today | overdue"
```

`dam category list` and `dam filter list` print the `[category.*]` and `[filter.*]` tables back, so
a client reads the catalogue from `dam` rather than from this file.

`stale` makes a read command pull that remote first when the last pull is older than the value.
Absent, reads never pull. `--no-pull` on the command line skips that pass for one invocation.

A `_command` inherits `dam`'s standard input. A vault CLI that prompts for a password works when
`dam` runs in a terminal and fails when a client spawns `dam` with standard input closed; a keychain
read works from both. Pick the command for how you will run it.

Storage is `~/.local/share/dam/dam.db`, one SQLite file in WAL mode, mode 0600.

## Performance

Reads are local SQLite queries. The targets, measured on a store of ten thousand objects:

- `dam ls <query> --json`: under 20 ms wall clock, process start included
- `dam status`: under 30 ms
- `dam pull` against Todoist with a sync token: bounded by the network, one request

The size ceiling is 16 MiB, one protocol line. A helper reads at most that much of any one upstream
answer and refuses a larger one by name, because an account whose full sync exceeds a line cannot be
delivered to `dam` at all.

The clients spawn `dam ... --json` per render and read the result. No daemon and no socket in
version one; the process start cost is measured before that is reconsidered.

## Errors

A refusal names the rule and the objects involved, then stops. A missing helper is named. A failed
push lists each failed mutation with the helper's reason. A parse failure in `edit -e` names the line
and reopens. No error message contains a token.

### The error document

`--json` and `--toon` replace the plain line on standard error with one document, so a client reads
the reason instead of matching substrings. Standard output stays empty on a failure.

```json
{"error": {"kind": "refused", "rule": "blocked",
 "message": "98d8780 cannot be completed: child a9db854 is open",
 "oids": ["98d878013fb0e026d37170e7ceed6707192ae99a",
          "a9db854060d1943ef9eb9f6d7a8ac0b1ace45d77"]}}
```

`kind` is one of `refused`, `store`, `helper`, `credential`, `parse`, `usage` and `cancelled`. An
editor failure is not among them: `-e` under a machine format is refused as `needs_an_editor`
rather than run, so no document is ever printed for an editor that died.

`rule` names the rule a refusal broke, one stable snake_case word per rule, and is null for every
other kind. `refused` alone is too coarse for a client: a task that is gone and a task that is
blocked are both refusals, and a list that should refresh itself on the first should show the
blockers on the second. The words are `blocked`, `cycle`, `exclusive_label`, `unknown_category`,
`no_such_object`, `no_working_object`, `no_such_remote`, `not_a_task`, `not_an_event`,
`not_completed`, `not_committed`, `dirty_on_pull`, `move_inside_itself`, `nothing_to_commit`,
`needs_an_answer`, `needs_an_editor`, `unresolved_conflicts` and `missing_credential`.

`message` is the sentence the human form prints after `dam: `, which is what carries the rule that
no token reaches an error. `oids` names the objects the message names, in full and in the order it
names them: for a blocked completion the task and then each blocker, for a cycle the task and then
the chain, for an ambiguous prefix every object it matched, and empty for a failure that names
none.

Under `--json` or `--toon`, standard error carries exactly that one document and nothing else, so a
client parses the whole stream rather than hunting a line in it. A run that succeeds writes nothing
there. A warning `dam` would print for the operator, such as a config file it had to tighten, is
returned as data and reaches the human answer instead.

Without `--json` or `--toon` nothing changes: the plain `dam: <message>` line, and the same exit
code. An argument clap rejects before `dam` runs prints clap's own usage text and exits 2, whatever
the format flag says.

A push failure is per mutation once mutations are being applied, and whole-push before that. The
read a helper needs to shape anything is the before: it fails the push with one reason, because no
mutation was attempted and there is nothing to report against one. From the first mutation onward a
failure belongs to the mutation it happened on, so the ones the remote already took are reported as
taken and `dam` marks exactly those pushed. The exception is a rate limit, which says the remote is
taking nothing more: the push stops there and `dam` marks nothing pushed.

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
    dam-domain/              identifiers, the object model, categories, completion rules,
                             recurrence, the query language, the change model; std and jiff only
    dam-application/         use cases over ports: new, done, edit, mv, rm, add, reset, commit,
                             status, ls, push, pull, resolve
    dam-protocol/            wire types, the line codec and the golden fixtures
    dam-adapters/            SQLite, TOML config, credentials, the helper process, the editor
    dam-cli/                 package `damnit`, binary `dam`: arguments, output, prompts
    dam-remote-todoist/      binary
    dam-remote-gcal/         binary, second, its own plan
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
- Automatic retry or backoff after a rate limit. A helper reports the rate limit and the retry time
  the remote named, `dam` stops the push without marking anything pushed, and the operator retries.

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
