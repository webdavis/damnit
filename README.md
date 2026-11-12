# damnit

`dam` keeps your tasks and calendar events in a local store and treats them the way git treats
files: a working layer you edit, a stage you `add` to, commits you `push` to a remote, and
`pull` to bring the remote's changes in. Todoist is the first remote.

## Install

    cargo install --git https://github.com/webdavis/damnit damnit
    cargo install --git https://github.com/webdavis/damnit dam-remote-todoist

`dam` finds a remote helper the way git does: a `dam-remote-<name>` binary on `PATH`.

## First run

The first `dam` run creates `~/.config/dam/config.toml`, or the file `DAM_CONFIG` names, readable
by you alone, with every setting commented out and explained. dam never rewrites a config that
exists, and a run with `DAM_TOKEN` set creates nothing.

    dam remote add todoist todoist::

Then give the helper a token, in `~/.config/dam/config.toml` under the table `dam remote add`
wrote:

    [remote.todoist]
    url = "todoist::"
    api_token_command = ["security", "find-generic-password", "-w", "-s", "Todoist API Token"]
    stale = "15m"

A credential takes one of three forms. `api_token` is the value, with `credentials = ["api_token"]`
in the same table. `api_token_command` is a command that prints it, run with your terminal so a
vault CLI can prompt. `api_token_env` names a variable. When more than one form is set, the value
wins over the command and the command over the variable. Any other key in the table fails with
exit 1.

`stale` makes reads pull first when the last pull is older than that. A failed pull that a read
started leaves a notice in `dam status`, and the read still answers. `deadline` (default `"60s"`)
bounds one helper answer; past it `dam` kills the helper and exits 1. Ctrl-C kills the helper,
leaves the store as it was and exits 3.

    dam pull

## Google Calendar

    cargo install --git https://github.com/webdavis/damnit dam-remote-gcal

That installs `dam-remote-gcal`, which dam runs, and `dam-gcal-sign-in`, which you run once.

Google needs an OAuth client of type "Desktop app" with the Google Calendar API enabled on its
project. Keep its client id and client secret in your vault, then sign in:

    security find-generic-password -w -s "Google dam client" | dam-gcal-sign-in --client-id <id>

The command prints a URL; open it, grant read-only access to your calendar events, and the refresh
token is printed once on standard output. Store it in your vault. dam asks Google for read-only
access.

Publish the OAuth consent screen ("In production") before you sign in. While its publishing status
is "Testing", Google issues refresh tokens that expire after 7 days, and every pull fails with
`invalid_grant` from then on until you sign in again. An app Google has not verified still works in
production; the consent page shows a warning you click through once.

`dam-gcal-sign-in` takes the client secret only on a piped standard input, never as an argument or
from a terminal. It listens for the browser on 127.0.0.1 only and waits as long as you take to open
the URL; once the browser connects, the redirect has 30 seconds to arrive. The token is written to
standard output and nowhere else; if that write fails, the command exits 1 and says to sign in
again. A command line it refuses exits 2 before anything is asked of Google; a refused consent or
exchange exits 1.

Then the remote. The address after `gcal::` names the calendars to read, comma-separated; empty
means your primary calendar:

    dam remote add gcal gcal::primary

    [remote.gcal]
    url = "gcal::primary"
    client_id_command = ["security", "find-generic-password", "-w", "-s", "Google dam client id"]
    client_secret_command = ["security", "find-generic-password", "-w", "-s", "Google dam client"]
    refresh_token_command = ["security", "find-generic-password", "-w", "-s", "Google dam refresh"]

A pull reads the week behind and the ninety days ahead, recurring events as their instances, and
files each event under a folder named after its calendar's title. The remote is read-only: it
declares no kinds, so `dam push` sends it nothing, and an event you make or change in dam stays in
dam. `path` narrows what a push sends, so it does nothing for this remote.

Pull it on a schedule: run `dam pull gcal` every few minutes from cron, launchd or a systemd timer.
Its credential commands must run without a terminal. Setting `stale` also keeps it fresh, but then
whichever read finds it stale waits on Google first.

## How Todoist maps

A Todoist project is a top-level path, a section is the next level down, and a task's parent tasks
are the levels below that: `Work/Now/milk/` is under the task `milk` in section `Now` of project
`Work`. A `/` in a Todoist name becomes `-`, a blank name reads as `untitled-<id>`, an archived
project comes in done, and dam's `p1` is Todoist's highest priority.

A push goes the other way. An object at the top level becomes a project. One level down it becomes
a task in that project, or a section when something lives under it or arrives under it in the same
push. Deeper levels name existing parent tasks by their text, and a path naming no Todoist project
fails that one object. A change counts as pushed only when Todoist accepts every command it became.
A rate limit stops the whole push and says how long Todoist asked to wait; any other failure stays
with the object it happened on. An error Todoist sends back is cut to one line of at most 500
characters before `dam status` shows it.

## Every day

    dam new "buy oat milk" --due tomorrow -p 1 --labels errand
    dam ls due:today
    dam done 3f2a9c1
    dam add -A
    dam commit -m "morning triage"
    dam push

`--due`, `--deadline`, `--start` and `--end` all read the same words, in any case: `today`,
`tomorrow`, a weekday (`mon` or `monday`, meaning its next occurrence after today, so naming today's
own weekday is a week out), `next mon` for that same day, `in 3 days`, `in 2 weeks`, `in 1 month`,
`YYYY-MM-DD`, and `YYYY-MM-DDTHH:MM` or an RFC 3339 time such as `2026-09-25T09:00:00-04:00`
where a time makes sense. A deadline is a whole day, so a time given to `--deadline` is dropped.
Anything else exits 2 with a message naming the accepted forms.

Every verb that takes an oid takes it whole or as a prefix of at least 4 hex characters that names
one object in the working layer. An object only the history holds, such as one you removed, takes
its full oid.

`dam edit <oid> -e` opens the object as a TOML template in `$VISUAL`, else `$EDITOR`, else `vi`: a
comment header naming the oid and kind, then one `key = value` line per field, where an empty
string means none. Emptying an optional field clears it, a key the object's kind does not have is
refused, and saving the template untouched cancels. A save that does not parse reopens the template
once with the error on its first line; a second one exits 2 and names the file that keeps your
edit. Under `--json` or `--toon` it refuses instead of opening an editor.

`dam edit <oid> --no-done` reopens a task you completed by mistake, and
`dam restore <oid>...` throws a working change away, back to the last commit and out of the stage.

`dam done` records when it ran, as `completed_at` on the task: RFC 3339 in UTC, null on an open
task, and cleared again by `--no-done`. A task completed in Todoist comes in with Todoist's time.

`dam status`, `dam diff`, `dam log` and `dam show` read the way their git namesakes do. `--json`
and `--toon` on any read give a program the same answer. A change's minus line is red and its plus
line green, unless output is not a terminal, `NO_COLOR` is set or `--no-color` is given.

`dam remote list` names every remote with when it was last pulled and last pushed: `pulled 4m ago`
in the line, `last_pull` and `last_push` as RFC 3339 under `--json`, null until that verb has
reached the remote once.

`--no-pull` answers from the local store: `dam ls`, `dam show`, `dam agenda` and `dam reminders`
otherwise pull a remote whose `stale` window has passed, and with the flag they spawn no helper at
all. Every other verb refuses the flag with exit 2.

## Planning a day

    dam edit 3f2a9c1 --estimate 5m
    dam ls estimate:max:5m
    dam edit 3f2a9c1 --focus today
    dam ls completed_at:today
    dam reminders --json

`--focus DAY` marks a task to work on first that day and records who set it and when. A day holds
three marks; a fourth exits 4, rule `focus_full`. `dam ls` lists today's marks first, in the order
they were set. The mark and the estimate stay in damnit and never reach a remote.

`dam day show [DAY]` prints a day's record: the plan, the proposals for the next day, the
decisions, each task's stall count and carry-forward, and `closed_at`. `dam day set` replaces a
day's record with the JSON or TOON document on stdin, or in the file `--request` names, and
`dam day list` lists the recorded days with when each closed. `docs/schemas/day.schema.json` is the document's schema.

`dam reminders` lists the reminders that fall due between `--from` and `--to`, by default from the
start of today until now: each one's `at`, its `anchor` (the due time or start) and
`minutes_before`, all times RFC 3339. `docs/schemas/reminders.schema.json` is its schema.

## The rules it keeps

- A task with open children or open dependencies is not done until they are; `--force` overrides,
  `--force --interactive` asks what to do with them, and `--children up|keep|into:<name>` with
  `--depends drop|keep` answers that question on the command line, for a client with no terminal.
- Labels can be grouped into categories in config; an exclusive category allows one value per
  object.
- A pull never overwrites work you have not committed, and never deletes on your behalf: an
  upstream removal shows in `dam status` until you `dam rm` it.
- No error message contains a token.

## Query language

    due:today | overdue
    path:work/ & effort:deep & !done
    @errand & p1

`estimate:max:5m`, `estimate:min:1h` and `estimate:none` read the estimate; `focus:` and
`completed_at:` take the same date words as `due:`.

`|` or, `&` and, `!` not, parentheses group. `@label`, `pN`, `key:value`, where a key is a
field's JSON key such as `labels:` or `attached:`; any other key is a category and the value must
be one of its values. Saved filters live under `[filter.<name>]` and run by name.

`dam filter list` prints the saved filters and `dam category list` prints the label categories with
their values and whether each is exclusive:
`{"filters": [{"name": "today", "query": "due:today | overdue"}]}` and
`{"categories": [{"name": "effort", "values": ["light", "admin", "deep"], "exclusive": true}]}`
under `--json`, an empty array when nothing is declared.

## Reading it from a program

A flag is named after the key it writes: a value `dam show --json` prints under `due` goes back
through `dam edit --due`, and `--no-due` clears it.

Every time under `--json` and `--toon` is RFC 3339 with its offset, such as
`2026-09-25T09:00:00-04:00`, and every all-day date is a plain `YYYY-MM-DD`.

Every answer has a JSON Schema in `docs/schemas/`, named after its command: `dam remote list`
answers by `remote-list.schema.json`. A command that writes also publishes
`<command>.request.schema.json`, whose keys are its flags without the dashes.
`object.schema.json` is a task or an event.

`--request FILE` gives a writing command its request as one JSON or TOON document instead of
flags, and `--request -` reads it from standard input. It means what the same flags mean. The
keys are those of the command's request schema: its arguments, such as `subject` or `oid`, its
flags, and `meta` as an object of details. Only global flags may come beside it, and `edit` cannot
open an editor from a request.

    dam new --request - <<< '{"subject": "buy oat milk", "due": "fri", "labels": ["errand"], "meta": {"session": "s1"}}'
    dam edit --request - <<< '{"oid": "98d878", "estimate": 30, "no_due": true}'
    dam done --request - <<< '{"oid": "98d878", "force": true, "children": "up"}'
    dam mv --request - <<< '{"oid": "98d878", "path": "work/"}'
    dam rm --request - <<< '{"oid": "98d878"}'
    dam token create --request - <<< '{"name": "tool", "allow": ["task:read"], "require": ["session"]}'
    dam day set --request day.json

`dam status --json` and `dam diff --json` answer in change documents, one per change:

    {"oid": "98d878...", "op": "update", "fields": ["subject", "due"], "kind": "task",
     "subject": "buy oat milk", "path": "work/", "labels": ["errand"], "done": false,
     "completed_at": null, "priority": 1, "due": "2026-09-25"}

`fields` names what the change touches. An update names what moved; a create names every field the
new object carries beyond its defaults; a delete names nothing and its row describes the object it
removed. The rest of the row is the state the change left behind, which is what a list renders.

Add `--full` to either command for `before` and `after`, the whole object on each side.

`dam agenda` lists events as intervals, for a program that schedules around busy time:

    {"events": [{"oid": "98d878...", "subject": "standup", "path": "me@example.com/",
     "labels": [], "status": "confirmed", "transparency": "busy", "all_day": false,
     "start": "2026-09-25T09:00:00-04:00", "end": "2026-09-25T09:30:00-04:00", "busy": true}]}

`start` and `end` are times in the local zone, or dates for an all-day event, `end` exclusive;
`busy` says whether the event holds time: not cancelled, shown busy, and not declined by the
attendee that stands for the calendar the event came from. Every event the window overlaps is
listed, busy or not; a recurring event made in dam is listed at its stored occurrence only. With no
flags the window runs from now to 24 hours later; `--from` and `--to` take the same words `--due`
does, and a query narrows it the way `dam ls` reads one. Keys are only ever added, never renamed or
retyped.

Like `ls`, `agenda` pulls a stale remote first unless given `--no-pull`.

`--max-age <remote>=<duration>` makes the answer depend on that remote being fresh. When its last
successful pull is older than the duration, or it has never pulled, `agenda` prints nothing on
standard output and exits 4, rule `stale_remote`, with one sentence on standard error. A failed
pull never counts as fresh. A program that polls with `dam pull gcal` scheduled every five minutes
runs:

    dam agenda --json --no-pull --max-age gcal=15m

Repeat the flag to bound more than one remote.

## Tokens

A tool runs `dam` with a token in `DAM_TOKEN`. Without `DAM_TOKEN` the caller is you. Each token is
a table in the config file, and its grants say what it may do:

    schema = 1

    [token.tool]
    value = "damk_v1_..."
    expires = 2027-05-11T00:00:00Z

    [[token.tool.grant]]
    allow = ["task:read", "event:read"]
    paths = ["work"]

`dam token create tool --allow task:read --allow event:read --paths work` writes such a table with
one grant and prints the token once. `--labels`, `--own`, `--remotes`, `--expires`, `--via` and
`--require` set the keys of the same name. `dam token list` shows each token's key id, grants,
conditions and standing, never its value. `dam token revoke tool` removes its tables and leaves the
rest of the file as it was. The schemas of their `--json` answers are
`docs/schemas/token-create.schema.json`, `token-list.schema.json` and `token-revoke.schema.json`.

A grant allows only what `allow` lists, and only on objects under one of `paths`, carrying one of
`labels`, or on the remotes `remotes` names. One grant must cover everything a command touches.
`own = true` covers only what the token made: a task it sent that you added, or its own waiting
item. It never stops a `new`. Ownership follows the name, so a new token given a revoked token's
name owns what the old one made. `not_before`, `expires` and `via` (default `["local"]`) bound when
and where the token works. `require = ["session", "model"]` refuses, with rule `missing_details`, a
request that does not pass each as `--meta session=...`. A token with `limits` is disabled until
limits are enforced.

The permissions are `task:` and `event:` `read`, `create`, `change`, `move` and `remove`,
`task:complete`, `pending:read`, `pending:accept`, `pending:reject`, `working:read`, `stage:read`,
`history:read`, `history:commit`, `remote:read`, `remote:pull`, `remote:push`, `conflict:read`,
`conflict:resolve` and `config:read`.

| Command | Needs |
|---|---|
| `ls`, `show`, `agenda`, `reminders` | `task:read` or `event:read`; objects outside every grant are left out, oids included |
| `new`, `edit`, `done` | `task:create` or `event:create`, `task:change` or `event:change`, `task:complete` |
| `status` | nothing for its own items; `stage:read`, `working:read`, `pending:read` and `conflict:read` for the rest, each row only for objects a grant covers |
| `diff` | `working:read` and `stage:read` |
| `log`, `show <commit>` | `history:read` |
| `category list`, `filter list` | `config:read` |
| `remote list`, `pull`, `push` | `remote:read`, `remote:pull`, `remote:push` |

A token's `new`, `edit` and `done` never touch your tasks. Each waits as an item in `dam status`,
with its hash, the task a change changes, the token's name and its age. A change shows the current
values on a minus line and the proposed ones on a plus line:

    Not staged:
      new      a1b2c3d  Call the dentist  inbox/  tool  2h
      changed  e4f5a6b  0c1d2e3  -Pay rent  due: 2026-11-13  tool  5m
                                 +Pay the rent  due: 2026-11-12

The tool's answer is `new`, `changed` or `duplicate` with the item's hash, and for a change the oid
of the task it changes. A new item's hash is the oid its task gets. Its grant must hold for the
task before and after the change.

`dam add <hash>` takes an item into your working layer and stage, and `dam commit` records it as
the token's change. Your own edit of the same task is staged and committed as yours. `dam add -A`
takes every item that applies. An item that changes a field you edited and have not saved stops
the add; settle it with `dam resolve <hash> --ours|--theirs`. An item that would make a dependency
cycle, or completes a task with open children or dependencies, is refused the way your own `edit`
or `done` would be. `dam reset` sends an added item back to waiting and keeps your own edits.
`dam rm <hash>` declines it, and the same change sent again is refused. A token may `dam rm` its
own waiting item.

Your own rows in `status` and `log` read `you` to you and `owner` to a token.

Every task and event has an `author`: the name of the token that made it, or yours, set as
`owner = "Ada"` at the top of the config. A request states its details with a repeatable
`--meta key=value`, such as harness, session, model, effort and version; any key is kept as given.
Each change keeps them with the host, working directory, repo, branch, time and channel. `dam show
<oid>` prints the author and the details of the change that made it, and `--json` and `--toon` carry
them as `author` and `details` on every object in `ls` and `show` and every change in `status`,
`diff` and `log`. A token reads the details of its own changes only. `dam ls --by mine`, `--by
agents` or `--by all` lists what you made, what tokens made, or both. damnit takes these values as
given; the token is the security check.

`dam new --id <text>` names the task with the tool's own id, which is not damnit's oid. The id is
one line of up to 200 characters. A repeat of `dam new` with the same token and id changes that
task: it waits as a change item, and only the fields that differ from the tool's last send change.
Your own repeat with a new `--path` or subject carries the task's children, as `dam mv` and
`dam edit` do. A token's repeat cannot change `--path`. Each token's ids, and your own, are kept
apart, and so are each session's when the request passes `--meta session=...`.

A token cannot run `mv`, `add`, `reset`, `restore`, `commit`, `remote add`, `resolve`, `day` or
`token`, `rm` an object, or set `--focus`. A read pulls a stale remote only when the token holds
`remote:pull` for it.

An unknown permission is skipped. An unknown key, or a reserved name such as `config:change`,
disables the token, and so does sharing a value with another token. A `schema` above 1 disables
every token. `dam status` warns about each.

## Errors

With `--json` or `--toon` a failure prints one document on standard error and nothing on standard
output:

    {"error": {"kind": "refused", "rule": "blocked",
     "message": "98d8780 has 1 open child: a9db854; pass --force to complete it anyway",
     "oids": ["98d878013fb0e026d37170e7ceed6707192ae99a",
              "a9db854060d1943ef9eb9f6d7a8ac0b1ace45d77"]}}

`docs/schemas/error.schema.json` is the JSON Schema of this document.
`kind` is one of `refused`, `store`, `helper`, `credential`, `editor`, `parse`, `usage`,
`cancelled` and `internal`, the last meaning a bug in `dam` itself.
`rule` names which rule a refusal broke, such as `blocked`, `no_such_object` or
`nothing_to_commit`, and is null for every other kind except an ambiguous oid prefix, a `usage`
failure with rule `ambiguous_prefix`. `oids` names the objects the message names, in the order it
names them.

Standard error carries that one document and nothing else, so parse the whole stream; a run that
succeeds writes nothing there. The exception is an argument clap rejects before `dam` runs, such as
a subcommand that does not exist: that prints clap's own usage text and exits 2, whatever the format
flag says. Without those flags you get the plain `dam: <message>` line instead.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | The command did what it was asked. |
| 1 | `dam` failed: a store, config, helper, editor or io failure, or a bug in `dam` itself. |
| 2 | The command line was wrong: an unknown argument, an unreadable value or query, or a bad oid. |
| 3 | Cancelled: you interrupted, a prompt could not be answered, or you saved an edit untouched. |
| 4 | `dam` refused by one of its own rules, and the message names the rule. |
| 5 | `dam` refused the token in `DAM_TOKEN`, and the message names the rule. |

Every rule `dam` keeps reports code 4, except a refused token, which reports code 5. A `--recurrence` rule `dam` cannot
read is the one flag value outside code 2: it exits 1, and its error document's kind is `parse`.
