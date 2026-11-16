# damnit

`dam` keeps your tasks and calendar events in a local store and treats them the way git treats
files: a working layer you edit, a stage you `add` to, commits you `push` to a remote, and
`pull` to bring the remote's changes in. Todoist is the first remote.

## Install

    cargo install --git https://github.com/webdavis/damnit damnit
    cargo install --git https://github.com/webdavis/damnit dam-remote-todoist

`dam` finds a remote helper the way git does: a `dam-remote-<name>` binary on `PATH`. Each remote's
helper is its own install: `dam-remote-todoist` for Todoist, and `dam-remote-gcal` for
[Google Calendar](#google-calendar).

## First run

The first `dam` run creates `~/.config/dam/config.lua`, or the file `--config` or `DAM_CONFIG`
names, with every setting commented out and explained. dam never rewrites a config that exists,
and a run with `DAM_TOKEN` set creates nothing.

    dam remote add todoist todoist::

The config is two Lua files in one folder, each returning a table. `config.lua` holds the settings
and never a secret. `secrets.lua` holds each remote's credentials and the tokens:

    -- config.lua
    return {
      remote = {
        todoist = { url = "todoist::" },
      },
    }

    -- secrets.lua
    return {
      schema = 1,
      remote = {
        todoist = { api_token = function() return os.getenv("TODOIST_API_TOKEN") end },
      },
    }

A credential is a string, or a function dam calls only when the remote is used. Any key dam does
not know fails with exit 1. `docs/examples/` holds one of each file, and a hook.
What `config.lua` prints goes to standard error, so it never mixes with a command's answer.

`dam config list` prints where the two files are and every setting dam is using, one row each: the
key, then the value, with `(default)` after a value dam supplies. A remote's credential shows under
`remote.<name>.credential` as only `hidden`, or `hidden (function)` for one dam calls.
A filter's `match` and a hook's `run` show as `function`, never their source, and a hook's
`file` names its file. A token shows
each of its settings by the key `secrets.lua` uses, never its value, or `disabled`. A setting with
no value and no default shows `not set`.
`docs/schemas/config-list.schema.json` is the schema of its `-o json` answer.
`dam config get <key>` prints the rows of one setting, or of every setting in one part, by the key
`dam config list` shows, such as `remote.todoist.deadline` or `remote.todoist`. A name may hold a
dot, and a key that can be read two ways is refused with both readings.
`dam config list` and `dam config get` run the top-level code of `secrets.lua` and call no
credential function. What that code prints is dropped, except through `io.open("/dev/stdout")`,
`io.popen` or `os.execute`. Their errors and warnings quote nothing from `secrets.lua` but a token's
name. Both warn of a token dam ignores or holds disabled, and `dam doctor` says why.

`dam config set <key> <value>` writes one setting into `config.lua`, and `dam config unset <key>`
removes it. The rest of the file stays as you wrote it. A list takes its items separated by commas.
A secret, a function and a table built in code are refused, with the line to write by hand.

`dam config print-default` prints the starter `config.lua` a first run writes, and writes no file.
`dam config` alone prints the help of these commands.

`dam doctor` reads this machine, says what is wrong and how to fix it, and contacts nothing.
It prints one row per check, then what needs attention, and creates no file.
`dam doctor --fix` creates the starter `config.lua` and the store where each is missing.
It changes no file that exists, and the report names each file it created.
`dam doctor --fix <check>...` creates only what the named checks cover.
`dam doctor <check>...` runs only the named checks: `config`, `database`, `helpers`, `credentials`,
`pushes`, `parents` and `sync`.
Its `config` check reads both files in full and lists each token warning under its row.
That row may quote an error raised in `secrets.lua`. Elsewhere only a credential function's own
error reaches you, and only when you run the command without `DAM_TOKEN`.
Its `database` check reads the store's version without changing the store, even when the config
does not read.
Its `helpers` check starts each remote's helper with no credential and asks only what it can do.
Its `credentials` check finds each credential the helper asks for in `secrets.lua`, and calls no
credential function.
Its `pushes` check warns of changes still owed to a remote, or sent without an answer.
Its `parents` check lists each object that sits under two same-named objects and does not know
which one is its parent.
Its `sync` check says whether background sync runs now, starts at login, how often, and how its
last run ended. A failed last run, or an enabled remote not run for three intervals, is an issue.
`helpers`, `credentials`, `pushes` and `sync` list one row for each remote under their own row.
`dam doctor` exits 1 when a check finds an issue.
`docs/schemas/doctor.schema.json` is the schema of its `-o json` answer.
`dam config validate` prints what `dam doctor config` prints, with `command` set to
`config validate`.

`dam test` makes each part do its real job once and says whether it worked.
It contacts each remote and runs credential functions, which may prompt.
`dam test <part>...` runs only the named parts: `credentials` and `filters`.
Its `credentials` part signs in to each remote and writes nothing.
Its `filters` part runs each saved filter once over the store, as `dam list --filter <name>` would, and
changes nothing. A failing filter names the object and the line of `config.lua` it failed on.
`dam test` prints the report `dam doctor` prints, with `command` set to `test`.
It exits 1 when a part finds an issue.
`docs/schemas/test.schema.json` is the schema of its `-o json` answer.

`deadline` (default `"60s"`) is the longest the helper may stay silent; past it `dam` kills the
helper and exits 1. A pull answers at once, so the deadline bounds the whole pull. Ctrl-C kills the
helper and exits 3. A push cut short keeps what the remote already answered, says how many it kept
and left unanswered, and sends the rest on the next push.
A push holds an object in conflict, and a removal or move that waits on one, and sends the rest.
It prints how many it held as `held`. The next push after `dam resolve` sends them.

    dam pull

## Sync

    dam sync run todoist

pulls from one remote, then pushes its committed changes. A conflict is left for `dam status`.
`sync = { interval = "15m", allowed = false }` on a remote sets background sync: `interval`
defaults to `15m` and is at least `1m`; `allowed = false` keeps it from ever starting. Nothing
syncs by itself.
`dam sync run` ignores `allowed`. `docs/schemas/sync-run.schema.json` is its `-o json` schema.
`dam sync run todoist --when-due` syncs only when `interval` has passed since the last sync, and
never when `allowed = false`; otherwise it prints nothing. `--interval 5m` replaces `interval`.
Each `dam sync run` notes when it ran and how it ended in the `background` folder beside the
database.

    dam sync start todoist      # now, until the next restart; --interval 5m for this run only
    dam sync stop todoist       # now; the next login still starts it if enabled
    dam sync enable todoist     # at every login; --now also starts it
    dam sync disable todoist    # no longer at login; --now also stops it

Each names one remote and is safe to repeat. `start` and `enable` refuse when `allowed = false`.
A token cannot run them. macOS uses launchd. Linux uses a systemd user timer; without
`loginctl enable-linger` it runs only while you are logged in. There, `start` refuses settings
other than an enabled remote's, which systemd would keep in force; disable it first. With
neither, `dam` refuses and prints the crontab line to add by hand. `docs/schemas/sync-start.schema.json` and its three siblings are their `-o json` schemas.

## Google Calendar

    cargo install --git https://github.com/webdavis/damnit dam-remote-gcal

That installs `dam-remote-gcal`, which dam runs, and `dam-gcal-sign-in`, which you run once.

Google needs an OAuth client of type "Desktop app" with the Google Calendar API enabled on its
project. Keep its client id and client secret in your password manager, then sign in, piping
the client secret:

    printf '%s' "$CLIENT_SECRET" | dam-gcal-sign-in --client-id <id>

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

Then its credentials, in `secrets.lua`:

    remote = {
      gcal = {
        client_id = function() return os.getenv("GCAL_CLIENT_ID") end,
        client_secret = function() return os.getenv("GCAL_CLIENT_SECRET") end,
        refresh_token = function() return os.getenv("GCAL_REFRESH_TOKEN") end,
      },
    },

A pull reads the week behind and the ninety days ahead, recurring events as their instances, and
files each event under a folder named after its calendar's title. The remote is read-only: it
declares no kinds, so `dam push` sends it nothing, and an event you make or change in dam stays in
dam. `path` and `except` narrow what a push sends, so they do nothing for this remote.

Pull it on a schedule: run `dam pull gcal` every few minutes from cron, launchd or a systemd timer.
Its credential functions must run without a terminal.

## How Todoist maps

A Todoist project is a top-level path, a section is the next level down, and a task's parent tasks
are the levels below that: `Work/Now/milk/` is under the task `milk` in section `Now` of project
`Work`. A `/` in a Todoist name becomes `-`, a blank name reads as `untitled-<id>`, an archived
project comes in done, and dam's `p1` is Todoist's highest priority.

A pull asks Todoist what changed since the last pull, so a task completed in the app comes in done
with its completion time and a task deleted there shows in `dam status` as removed. An open task
that Todoist no longer lists, for a reason that pull cannot tell, is named in `dam status`; settle
it with `dam done <oid>` or `dam rm <oid>`.

A push goes the other way. An object at the top level becomes a project. One level down it becomes
a task in that project, or a section when something lives under it or arrives under it in the same
push. Deeper levels name existing parent tasks by their text, and a path naming no Todoist project
fails that one object. A change counts as pushed only when Todoist accepts every command it became.
A rate limit stops the whole push and says how long Todoist asked to wait; any other failure stays
with the object it happened on. An error Todoist sends back is cut to one line of at most 500
characters before `dam status` shows it.

A repeat rule goes to Todoist as text, in Todoist's documented words where it has them, and comes
back as the same rule:

| dam | Todoist |
| --- | --- |
| `every week on mon,fri` | `every mon, fri` |
| `every month on 1,15` | `every 1, 15` |
| `every 2 weeks on fri` | `every other fri` |
| `every month on 1tue,-1fri` | `every 1st tue, last fri` |

`every!` and `until DATE` stay as they are. Every other rule is sent as dam writes it, such as
`every 3 days`, `every 2 weeks on mon,fri`, `every 2 months on 1,15` and `every year`. A rule with
more months between weekdays of the month, such as `every 2 months on 1tue`, or counted back past
the last weekday, such as `-2fri`, fails that one object.

The task's due date and time travel in the same text, such as
`every mon, fri at 20:00 starting 4 Dec 2026`, so Todoist starts the rule on dam's date. A due with
a time goes as that instant with its time zone, so Todoist keeps the clock time in that zone.

## Every day

    dam new "buy oat milk" --due tomorrow -p 1 --labels errand
    dam list --due today
    dam done 3f2a9c1
    dam add -A
    dam commit -m "morning triage"
    dam push

`--labels errand,home` adds two labels; the flag also repeats.
`dam new` takes every field `dam edit` sets: `--recurrence`, `--estimate`, `--depends`,
`--attached` and `--focus` for a task, `--location` for an event.

`dam list` lists the tasks that are not done. Each field is a flag of the same name, such as
`--due`, `--labels`, `--completed-at` or `--start`; `dam list --help` lists them all. Every flag
repeats and each must hold. A comma list holds when any of its values does, `not:` when none does,
and `none` when the field is empty. `--any FIELD=VALUE` holds when any of the `--any` flags does.
`--done any` and `--kind any` lift the defaults, and a field only events have lists events.
`--sort FIELD[:desc]` orders the rows by any field, empty values last; `--limit N` and
`--offset N` keep a slice of them.
A value dam cannot read exits 2 and names the shapes the field takes and the command to run.
Under `-o json`, each row also carries `category`, the label of each group it holds, and a task's
`overdue` and `blocked`, read the same way the flags of those names select, `days_until_due` and
`days_until_deadline`, whole days from today, negative once past, and `children_open`, its children
that are tasks not done, whatever the filter lists. `dam show`, `dam new` and `dam edit` carry
`category`, `overdue` and the two day counts. The day counts and `children_open` cannot be selected
or sorted by.

    dam list --labels bug,ui --due ..today
    dam list --sort priority,due:desc --limit 10
    dam list --done --completed-at week
    dam list --path work/ --any labels=urgent --any due=..today
    dam list --filter today --author not:owner

`dam push --preview [remote]` shows what a push would send, hold and skip for each remote, and
starts no helper and writes nothing. It needs one `dam pull` or `dam push` with that remote after
upgrading; until then it says the remote is not known.

`--due`, `--deadline`, `--start`, `--end` and `--focus` all read the same words, in any case:
`today`, `today+3` or `today-1` for that many days on or back, `now+2h` for a time from now,
`tomorrow`, a weekday (`mon` or `monday`, meaning its next occurrence after today, so naming today's
own weekday is a week out), `next mon` for that same day, `in 3 days`, `in 2 weeks`, `in 1 month`,
`YYYY-MM-DD`, and `YYYY-MM-DDTHH:MM` or an RFC 3339 time such as `2026-09-25T09:00:00-04:00`
where a time makes sense. A deadline is a whole day, so a time given to `--deadline` is dropped.
A date to set is one day or instant, so a period such as `2026-11` or `week`, a span or `on:thu`
exits 2 with a message naming the accepted forms.

`--recurrence` takes `every [N] day|week|month|year`, then `on mon,thu` for a weekly rule or, for a
monthly one, `on 1,15` or `on 1tue,-1fri` (the first Tuesday and the last Friday; `1` to `5` and
`-1` to `-5`, and a month without that day is skipped), then `until YYYY-MM-DD`. `every!` counts
from the day the task is done.

Every verb that takes an oid takes it whole or as a prefix of at least 4 hex characters that names
one object in the working layer. An object only the history holds, such as one you removed, takes
its full oid.

`--parent <oid>` on `dam new` and `dam mv` puts the object in that object, and its path defaults to
that object's folder. When two objects of the same name hold a path, `dam new` and `dam mv` to it
are refused, rule `several_hold_the_path`, until `--parent` names one.

`dam edit <oid> -e` opens the object as a TOML template in `$VISUAL`, else `$EDITOR`, else `vi`: a
comment header naming the oid and kind, then one `key = value` line per field, where an empty
string means none. Emptying an optional field clears it, a key the object's kind does not have is
refused, and saving the template untouched cancels. A date in it reads the way `--due` reads one.
A save that does not parse reopens the template
once with the error on its first line; a second one exits 2 and names the file that keeps your
edit. Under `-o` it refuses instead of opening an editor.

`dam edit`, `dam done`, `dam mv` and `dam rm` take several oids, `-` or `--filter NAME` in place
of one oid, and change the whole set or nothing:

    dam list --path work/ --due ..today | dam done - --preview
    dam edit --filter postponed --due mon --expect 12
    dam mv 98d8 a9db work/clients/

`-` reads the ids from standard input: the rows `dam list` prints, or its `-o json` document. They
are a set even when one comes in; when none come in, nothing changes. `--preview` prints the
changes and writes nothing; it works on one oid too. `--expect N` refuses, changing nothing, unless
the set holds N objects. With `-` or `--filter`, `dam mv` takes its path from `--path`. The changes land unstaged, so `dam restore <oid>...` throws them away. The
answer is `docs/schemas/changes.schema.json`.

`dam new --preview` answers the same changes document for the one create, and a project task
`--project` would create, and writes nothing.

`dam` never asks before a large change unless `confirm_above = 10` is in `config.lua`. Then a
`--filter` run over more than 10 objects asks `change 42 objects? [y/N]`, and no exits 3. Without
a terminal, or with ids piped in through `-`, it needs `--expect 42` instead. Named oids,
`--preview`, `--expect` and tokens are never asked.

A token's set run waits as one item per object. With `--as-one` it waits as one set: `dam add
<set hash>` takes it whole or not at all, and `dam rm <set hash>` declines it whole.

`dam edit <oid> --no-done` reopens a task you completed by mistake, and
`dam restore <oid>...` throws a working change away, back to the last commit and out of the stage;
`--preview` shows what it would throw away and writes nothing.

`dam revert <commit> [<oid>...]` sets back what a commit changed, or only the named objects' part.
Each field goes back to its value before the commit; a field changed again since is kept and named.
A task the commit created goes only when nothing about it changed since.
A task the commit removed comes back.
The result is a working change to read in `dam diff`, then commit or `dam restore`.
It refuses a commit `dam pull` made, rule `pulled_commit`, and an oid it did not change, rule
`not_in_commit`.

`dam done` records when it ran, as `completed_at` on the task: RFC 3339 in UTC, null on an open
task, and cleared again by `--no-done`. A task completed in Todoist comes in with Todoist's time.

`dam status`, `dam diff`, `dam log` and `dam show` read the way their git namesakes do. `-o json`,
`-o toon` and `-o yaml` on any read give a program the same answer. A change's minus line is red
and its plus line green, unless output is not a terminal, `NO_COLOR` is set or `--no-color` is given.

`dam remote list` names every remote with when it was last pulled and last pushed: `pulled 4m ago`
in the line, `last_pull` and `last_push` as RFC 3339 under `-o json`, null until that verb has
reached the remote once.

`dam list`, `dam show`, `dam agenda` and `dam reminders` read only the local store and never reach a
remote.

## Planning a day

    dam edit 3f2a9c1 --estimate 5m
    dam list --estimate ..5m
    dam edit 3f2a9c1 --focus today
    dam list --completed-at today
    dam reminders -o json

`--focus DAY` marks a task to work on first that day and records who set it and when. A day holds
three marks; a fourth exits 4, rule `focus_full`. `dam list` lists today's marks first, in the order
they were set. The mark and the estimate stay in damnit and never reach a remote.

`dam day show [DAY]` prints a day's record: the plan, the proposals for the next day, the
decisions, each task's stall count and carry-forward, and `closed_at`. `dam day set` replaces a
day's record with the JSON or TOON document on stdin, or in the file `--request` names, and
`dam day list` lists the recorded days with when each closed. `docs/schemas/day.schema.json` is the document's schema.

`dam reminders` lists the reminders that fall due between `--from` and `--to`, by default from the
start of today until now: each one's `at`, its `anchor` (the due time or start) and
`minutes_before`, all times RFC 3339. `--from` and `--to` take what they take in `dam agenda`. `docs/schemas/reminders.schema.json` is its schema.

## Projects

`dam init` makes the folder it runs in a project. It writes `.dam.lua` there, which holds where the
project's tasks go, and creates the task that holds that path when nothing holds it yet:

```lua
return { path = "app/" }
```

`--path` names the path; the default is the folder's name. Commit `.dam.lua` so every clone shares
it. When the path is already there, `dam init` joins it and says how many tasks it holds. Where
`.dam.lua` is already there, as in a fresh clone, `dam init` creates what is missing, the project's
task and each of its `sections`, and says what it created; when nothing is missing it changes nothing. Inside another project's folder it makes a project of
its own: the nearest `.dam.lua` at or above where you stand is the one `dam` follows.
`docs/schemas/init.schema.json` is its `-o json` schema.

Inside a project, `dam new` puts the task under the project's path: `dam new "x" --path cli/` in a
project at `app/` lands at `app/cli/`. With `--parent` and no `--path` the task goes where its
parent is.

The nearest `.dam.lua` at or above the folder decides the project; a project inside another uses
only its own file.

`dam list`, `dam agenda` and `dam reminders` list only the project there: `dam list` in that project
shows what `dam list --path app/` shows elsewhere, and a `--path` you type starts from the project. A
project's filter reads its `path` from the project, yours from the root, and both are narrowed to the project. Listings print full paths.
`--global` on these four commands reaches every task and reads a path from the root.

`dam edit`, `dam done`, `dam mv` and `dam rm` with `--filter` change only what `dam list` lists
there: the project's tasks, its filters found before yours, its categories in the filter.
`--global` picks from every task with your own filters and categories; `--project <path>` picks
that project's. Oids, typed or piped in, are never narrowed: `dam done 98d8` works on any task
from anywhere. When `.dam.lua` does not load, a `--filter` run is refused unless `--global` or `--project`
is given. On `dam mv`, `--global` and `--project` also decide where the destination path starts.
`status`, `push`, `pull`, `log` and `show` always cover everything.

`--project <path>` on those four commands works from any folder, in a project or not, and stands in
for the project you are in: `dam list --project pns/hooks/`, `dam new "x" --project pns/ --path cli/`.
It takes a path of any depth. `--global` and `--project` together are a usage error. When no task
holds that path yet, `dam new --project` creates it first and says so on standard error; with `-o json`
the answer carries `project_created`, its `oid` and `path`.
`dam config list` inside a project shows the project's `.dam.lua` and each setting it holds,
marked `(from the project)`; with `-o json` they are under `project`.
Inside a project, `dam config set` writes a key `.dam.lua` holds into `.dam.lua`, and
`dam config unset` removes one but `path`, keeping the rest of the file, and refuses a value that would leave the file unable to load. A key only your `config.lua`
holds is refused there; `--global` writes `config.lua` instead. `project.hooks` never goes into
`.dam.lua`.

`.dam.lua` may hold more rules for the project; `docs/lua.md` lists them. They apply only to
commands run inside the project without `--global` or `--project`.
`new_task = { labels = { "app" }, priority = 2 }` fills what a flag and the template leave unset.
With `required = { "due" }`, `dam new` refuses a task without a due date and names `--due`.
`list = { priority = 1 }` is what `dam list` shows there when you type no selection flag.
`filter = { hot = { priority = 1 } }` is found before your own filter named `hot` in `dam list`,
`dam agenda` and the `--filter` of `dam edit`, `dam done`, `dam mv` and `dam rm`, and
`dam filter list` there marks it `(the project's)`. Inside a project, `dam filter create|remove`
and `dam category create|remove` change `.dam.lua`, and `--global` changes your `config.lua`.
`category = { area = { values = { "ui", "db" }, exclusive = true } }` adds label categories, as
`config.lua` writes them, to `dam new`, `dam edit` and `--category` there. Inside the
project, a project category sets aside whole any of yours that shares its name or a label; `dam edit`
judges a task under the project's path that way, and a task outside that path, or any task with
`--global`, by yours alone. `dam doctor` names each one set aside, and `dam category list` there
marks each category the project's, yours, or yours set aside.
`sections = { "bugs", "features" }` names standard sections that `dam init` creates under the path.
With `branch_label = true`, `dam new` adds the checked-out git branch's name to the new task's
labels, beside the ones from `--labels`, a template or the project's defaults. A detached head or no
repository adds nothing, and a branch name that is not a usable label adds nothing and prints one
warning.
`done_children = "up"` decides what `dam done --force` does with the open children of a task under
the path when `--children` is not given; it takes the words `--children` takes. Without `--force`
the refusal stands.
With
`minimum_dam = "0.3.0"` an older `dam` refuses the file and names the version it needs.

`dam new --template bug "crash on pull"` starts the task from a template: the project's
`.damtemplates/bug.lua` first, then `templates/bug.lua` beside your `config.lua`. Each flag you give
replaces the template's value, and `--labels` replaces its labels. A name found in neither is
refused with the names that exist; a token is told only that the name is not there. See
`docs/lua.md` for what a template holds.

When `.dam.lua` does not load, `dam new`, `dam list`, `dam agenda` and `dam reminders` print the error
and refuse, since they cannot tell where a task belongs; `--global` and `--project` do not read the
file. Every other command carries on with one warning on standard error, and `dam doctor config`
reports it.

## The rules it keeps

- A task with open children or open dependencies is not done until they are; `--force` overrides,
  `--force --interactive` asks what to do with them, and `--children up|keep|into:<name>` with
  `--depends drop|keep` answers that question on the command line, for a client with no terminal.
- Labels can be grouped into categories in `config.lua`; an exclusive category allows one value per
  object.
- `--category effort=deep` on `dam new` and `dam edit` gives the object the label `deep` of the
  group `effort`, and `--category effort=none` takes the group's label off. A group or label not
  declared where the command runs is refused. A second label of an exclusive group is refused, as
  with `--labels`; take the first off with `--category effort=none` in the same command.
- A pull never overwrites work you have not committed, and never deletes on your behalf: an
  upstream removal shows in `dam status` until you `dam rm` it.
- When the remote changed an object you changed or removed and have not committed, the pull marks a
  conflict for it in `dam status`, and `dam resolve` settles it.
- No error message contains a token.

## Saved filters

A saved filter is a table under `filter.<name>` in `config.lua`: each key is a `dam list` flag and
its value the text typed after it. A number or `true`/`false` may be bare, a list repeats the flag,
and `any` holds tables of which at least one must hold:

    filter = {
      today = { due = "..today" },
      review = { labels = { "bug", "not:waiting" } },
      hot = { any = { { labels = "urgent" }, { due = "..today" } } },
    }

`--filter NAME` adds its fields to the flags typed, and the defaults apply once over both, so
`dam list --filter today --done` lists the done tasks it selects. `in:NAME` asks what
`dam list --filter NAME` lists, and `on:NAME` the days its events take place. A filter may
also hold a Lua `match` function, which narrows what the fields list; `docs/lua.md` describes it.
A filter written as `query` text, or one that reaches itself, stops the config from loading; the
message shows the table to write. `on:NAME` knows only the events dam has pulled; when that left
out something `dam list` would otherwise list, it says how many on standard error.

`dam filter list` prints each saved filter as the flags that select it, and
`dam category list` prints the label categories with their values and whether each is exclusive:
`{"filters": [{"name": "today", "table": {"due": "..today"}}]}` and
`{"categories": [{"name": "effort", "values": ["light", "admin", "deep"], "exclusive": true}]}`
under `-o json`, an empty array when nothing is declared.

`dam label list` prints every label the tasks and events carry, done ones too, with how many
carry each: `{"labels": [{"name": "urgent", "count": 3}]}` under `-o json`. Inside a project
it counts only the project's objects; `--global` counts all. `dam path list` prints every path
that holds one, and each parent path, with how many sit at or under it:
`{"paths": [{"path": "work/", "count": 5}]}`.

`dam filter create <name>` with `dam list` selection flags saves a filter, such as
`dam filter create today --due ..today`, and `dam filter remove <name>`
removes one. `dam category create <name> --values a,b --exclusive` and `dam category remove <name>`
do the same for a category. A removed category's labels stay on tasks as free labels. A filter
with a `match` function is changed by hand in `config.lua`.

`dam category migrate --map quick=light --map hard=deep` renames labels on every task and event
that carries them, all or none, and takes `--preview` and `--expect N`. It leaves `config.lua` as
it is and lists each category and saved filter that still names an old label, with the
command that fixes it. `--update-config` also writes those into `config.lua`.

## Hooks

A hook runs when your `dam new`, `dam edit`, `dam done`, `dam add` or `dam resolve --theirs`
changes a task. Each hook is its own file in the `hooks` folder beside `config.lua`, named for its
event, such as `hooks/task_done.lua`:

    return function(task)
      print("done: " .. task.subject)
    end

The events are `task_new`, `task_changed` and `task_done`. A hook may return a table of fields,
as `dam edit --request` takes them, and dam sets them on that task. What a hook prints goes to
standard error. A failing hook warns, or with `on_failure = "refuse"` stops the command, which
then changes nothing and exits 4. A `dam` run with `DAM_HOOK` set, as a hook's own commands are,
exits 4 at once. `dam log` names the hook that changed a task. `docs/lua.md` describes hooks.

`hooks/task_pulled.lua` runs after a pull, as `run(task, before, pull)`, for each task the pull created
or changed. dam commits its changes after the pull's commit, or with `changes = "working"` leaves
them as working changes. It never runs on the first pull from a remote, and a failing one never
undoes the pull.
It has 60 seconds in total for one pull, and Ctrl-C stops it and keeps the pull.
`dam status` notes the tasks it failed on or did not reach.
`dam pull` answers what it did under `task_pulled`.

`dam hook list` prints each hook file with its event and its `on_failure` or `changes`, and warns
why a file does not load.

`dam hook run <hook>` calls one hook for each task, done or not, or each task the selection flags of
`dam list` select, such as `--labels waiting` or `--filter NAME`, as if each were
new, and leaves its changes as working changes to read in `dam diff`. Use it for tasks no hook ran
for: a first pull, a task a hook failed on, a task older than the rule. A rule that compares before
and after does nothing here, a hook with a side effect fires once per task, and nothing retries by
itself. It answers `ran`, `changed`, `failed` and `not_reached`.

A project may ship hooks in `.damhooks/`. They stay off, never loaded, until you turn them on:
`dam config set project.hooks true` inside the project for this clone, or
`dam config set --global project.hooks true` for every project. Turning them on runs that
repository's code as you. A clone's hooks stop running when their files change, until you look
and turn them on again; `--global` is a yes to every project's files as they are, now and after
every pull. They run after yours, only for tasks under the
project's path and only for commands run inside it. `docs/lua.md` says more.

## Reading it from a program

`-o` (`--output`) prints the answer as `json`, `toon` or `yaml`, the same data in three forms;
`toon` is compact for language models.

A flag is named after the key it writes: a value `dam show -o json` prints under `due` goes back
through `dam edit --due`, and `--no-due` clears it.

`--quiet` (`-q`) prints only errors; under `-o` the answer still prints.

A slow step shows a progress line on standard error when it is a terminal. `--progress` shows it
even when it is not, as one start and one finished record per step; `--no-progress` hides it.
These steps report progress:

- reading a credential that a function in `secrets.lua` computes
- waiting for each remote's answer in a pull
- sending changes to each remote in a push
- the `task_new`, `task_changed`, `task_done` and `task_pulled` hooks, and `dam hook run`
- signing in to each remote in `dam test credentials`
- trying each saved filter in `dam test filters`
- each helper check in `dam doctor`
- upgrading the store on the first run after a new version

A wait the remote asks for to slow down shows as the pull or push step it belongs to.

Every time under `-o` is RFC 3339 with its offset, such as
`2026-09-25T09:00:00-04:00`, and every all-day date is a plain `YYYY-MM-DD`.

Every answer has a JSON Schema in `docs/schemas/`, named after its command, and it describes the
answer in all three forms: `dam remote list` answers by `remote-list.schema.json`. A command that writes also publishes
`<command>.request.schema.json`, whose keys are its flags without the dashes.
`object.schema.json` is a task or an event.

`--request FILE` gives a writing command its request as one JSON or TOON document instead of
flags, and `--request -` reads it from standard input. It means what the same flags mean. The
keys are those of the command's request schema: its arguments, such as `subject` or `oid`, its
flags, and `meta` as an object of details. Only global flags may come beside it, and `edit` cannot
open an editor from a request.

`dam list --request` takes its selection as one table, by `list.request.schema.json`: each key is a
field as the rows print it, such as `completed_at`, its value the text typed after the flag, a list
the flag repeated, and `any` a list of tables that may hold several fields and their own `any`.
`sort`, `limit` and `offset` sit beside them.

    dam list --request - <<< '{"path": "work/", "any": [{"labels": "urgent", "priority": 1}, {"due": "..today"}], "sort": "due"}'
    dam new --request - <<< '{"subject": "buy oat milk", "due": "fri", "labels": ["errand"], "meta": {"session": "s1"}}'
    dam edit --request - <<< '{"oid": "98d878", "estimate": 30, "no_due": true}'
    dam done --request - <<< '{"oid": "98d878", "force": true, "children": "up"}'
    dam mv --request - <<< '{"oid": "98d878", "path": "work/"}'
    dam rm --request - <<< '{"oid": "98d878"}'
    dam token create --request - <<< '{"name": "tool", "allow": ["task:read"], "require": ["session"]}'
    dam day set --request day.json

`dam status -o json` and `dam diff -o json` answer in change documents, one per change:

    {"oid": "98d878...", "op": "update", "fields": ["subject", "due"], "kind": "task",
     "subject": "buy oat milk", "path": "work/", "labels": ["errand"], "done": false,
     "completed_at": null, "priority": 1, "due": "2026-09-25"}

`fields` names what the change touches. An update names what moved; a create names every field the
new object carries beyond its defaults; a delete names nothing and its row describes the object it
removed. The rest of the row is the state the change left behind, which is what a list renders.

Add `--full` to either command for `before` and `after`, the whole object on each side.

`dam agenda` lists events as intervals, for a program that schedules around busy time:

    {"from": "2026-09-25", "to": "2026-09-25",
     "events": [{"oid": "98d878...", "subject": "standup", "path": "me@example.com/",
     "labels": [], "status": "confirmed", "transparency": "busy", "all_day": false,
     "start": "2026-09-25T09:00:00-04:00", "end": "2026-09-25T09:30:00-04:00", "busy": true,
     "attendees": [{"email": "ada@example.com", "response": "accepted"}],
     "organizer": {"email": "ada@example.com"}, "day": "2026-09-25", "weekday": "fri"}]}

`from` and `to` are the first and last day of the window. `day` is the day a row is listed under,
an event's start day or a task's due day, and `weekday` its weekday, both in the local zone.
`start` and `end` are times in the local zone, or dates for an all-day event, `end` exclusive;
`busy` says whether the event holds time: not cancelled, shown busy, and not declined by the
attendee that stands for the calendar the event came from. `attendees`, `organizer`, `location` and
`conference` read as they do in `dam show`; `attendees` is always a list, and the others appear only
when the event has them. Every event the window overlaps is listed, busy or not; a recurring event
made in dam is listed at its stored occurrence only. With no flags the window runs from now to 24
hours later. `--from` and `--to` take a day, an instant or an anchor such as `today+7`, `week+1` or
`now+2h`; a day or period as `--to` counts whole, so `--to today` runs to midnight, and an instant
there is not included. The selection flags of `dam list`, such as `--labels work` or
`--filter NAME`, narrow the events, and `--request` reads them as JSON. `--kind any` adds
`tasks`: the tasks not done whose due falls on a day of the window in the local zone, by day, each
as `dam list` prints it. `--kind task` lists those tasks alone. Keys are only ever added, never
renamed or retyped.

`--max-age <remote>=<duration>` makes the answer depend on that remote being fresh. When its last
successful pull is older than the duration, or it has never pulled, `agenda` prints nothing on
standard output and exits 4, rule `stale_remote`, with one sentence on standard error. A failed
pull never counts as fresh. A program that polls with `dam pull gcal` scheduled every five minutes
runs:

    dam agenda -o json --max-age gcal=15m

Repeat the flag to bound more than one remote.

## Tokens

A tool runs `dam` with a token in `DAM_TOKEN`. Without `DAM_TOKEN` the caller is you.
[docs/agents.md](docs/agents.md) tells a tool or an agent what to pass. Each token is a table in
`secrets.lua`, and its grants say what it may do:

    return {
      schema = 1,
      token = {
        tool = {
          value = "damk_v1_...",
          expires = "2027-05-11T00:00:00Z",
          grant = {
            { allow = { "task:read", "event:read" }, paths = { "work" } },
          },
        },
      },
    }

`dam token create tool --allow task:read --allow event:read --paths work` writes such a table with
one grant and prints the token once. `--labels`, `--own`, `--remotes`, `--expires`, `--via` and
`--require` set the keys of the same name. dam edits only a token table written out in full, and
refuses before it makes a token when it cannot. `dam token list` shows each token's key id,
grants, conditions and standing, never its value. `dam token revoke tool` removes its table and
leaves the rest of the file as it was. The schemas of their `-o json` answers are
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
| `list`, `show`, `agenda`, `reminders`, `label list`, `path list` | `task:read` or `event:read`; objects outside every grant are left out, oids included |
| `new`, `edit`, `done` | `task:create` or `event:create`, `task:change` or `event:change`, `task:complete` |
| `rm` | nothing for its own waiting item; `task:remove` or `event:remove` to send a removal |
| `status` | nothing for its own items; `stage:read`, `working:read`, `pending:read` and `conflict:read` for the rest, each row only for objects a grant covers |
| `diff` | `working:read` and `stage:read`; a row only when `task:read` or `event:read` covers every state of its object |
| `log`, `show <commit>` | `history:read` |
| `category list`, `filter list` | `config:read` |
| `remote list`, `pull`, `push` | `remote:read`, `remote:pull`, `remote:push`; `conflict:read` for the count `push` held |

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
takes every item that applies. `dam add --preview` shows what it would stage and which items it
would take, and stages nothing, takes no item and runs no hook. An item that changes a field you edited and have not saved stops
the add; settle it with `dam resolve <hash> --ours|--theirs`, where `--theirs` runs the hooks
`dam add` would. An item that would make a dependency cycle, or completes a task with open children or dependencies, is refused the way your own `edit`
or `done` would be. `dam reset` sends an added item back to waiting and keeps your own edits.
`dam rm <hash>` declines it, and the same change sent again is refused until `dam reset <hash>` puts
the item back to waiting. A token may `dam rm` its own waiting item. A token's `dam rm <oid>` on one
of your tasks needs `task:remove` and waits as a removal item. A token's `dam rm` by hash, oid or
`--id` answers `withdrawn`, `waiting`, `declined` or `nothing`. Adding a removal item does what your
`dam rm` does: children move up and other tasks stop depending on it. Those changes are staged as
yours, and `dam status` counts them first. If you have removed the task yourself, `dam add` settles
the removal item and stages nothing. `dam reset <hash>` sets back the other tasks an added item
moved or changed. A task you changed since `dam add` stays as it is and is named as `kept`.

Your own rows in `status` and `log` read `you` to you and `owner` to a token.

Every task and event has an `author`: the name of the token that made it, or yours, set as
`owner = "Ada"` in `config.lua`. A request states its details with a repeatable
`--meta key=value`, such as harness, session, model, effort and version; any key is kept as given.
Each change keeps them with the host, working directory, repo, branch, time and channel. `dam show
<oid>` prints the author and the details of the change that made it, and `-o` carries
them as `author` and `details` on every object in `list` and `show` and every change in `status`,
`diff` and `log`. A token reads the details of its own changes only. `dam list --author NAME`
lists what NAME made, and `--author not:NAME` the rest. damnit takes these values as
given; the token is the security check.

`dam new --id <text>` names the task with the tool's own id, which is not damnit's oid. The id is
one line of up to 200 characters. A repeat of `dam new` with the same token and id changes that
task: it waits as a change item, and only the fields that differ from the tool's last send change.
Your own repeat with a new `--path` or subject carries the task's children, as `dam mv` and
`dam edit` do. A token's repeat cannot change `--path`. Each token's ids, and your own, are kept
apart, and so are each session's when the request passes `--meta session=...`.
`dam rm --id <text>` takes the task back by the same id. A waiting item is withdrawn; a task you
added gets a removal item. It answers `withdrawn`, `waiting`, `declined` or `nothing`, each with
exit 0. Your own `dam rm --id` removes your task, or answers `nothing`.

A token cannot run `mv`, `add`, `reset`, `restore`, `commit`, `remote add`, `resolve`, `day`,
`config`, `hook`, `token`, or `create` or `remove` under `filter` and `category`, or set `--focus`.

An unknown permission is skipped. An unknown key, or a reserved name such as `config:change`,
disables the token, and so does sharing a value with another token. A `schema` above 1 disables
every token. `dam status` warns about each.

## A config your dotfiles manage

`config.lua` holds no secret, so track or template it like any other file.

1. Run `dam token create tool --allow task:read`. It writes the token's table to `secrets.lua` and
   prints the token once.
2. Keep the token in your password manager.
3. Track `secrets.lua`, or have your template write it with that value.
4. Give the tool the same value as `DAM_TOKEN`.

## Neovim

The Neovim plugin lives in `nvim/` and is being built. Today it gives `:Dam` (the status window),
`:Dam list [view]`, `:Dam messages`, which shows each error dam gave with the command it suggests, and
`:Dam cancel`, which stops every running dam call.

`:Dam list` shows what `dam list` lists in the window's folder; `r` reads it again and `g?`
shows the keys. A view is a saved filter's name, or a name from the `views` option:

```lua
require("damnit").setup({
  views = {
    soon = { due = "..today+3", sort = "due" },
    everywhere = { global = true },
    errands = "errands",
    places = { sort = "path", group = true },
    week = {
      { title = "Urgent", labels = "urgent" },
      { title = "This week", due = "today..today+6", labels = "not:urgent" },
    },
  },
})
```

A table is keyed like the flags of `dam list` and sent as `dam list --request -`; a text names
a saved filter. `group = true` puts a header wherever the first `sort` key's value changes. A list
of tables is a view of sections, one `dam list` each, under a header with its `title` and count.
`:Dam today` is such a view: Overdue, Today, Next 7 days and No date.

A row shows its checkbox, priority and subject, with dam's labels, category, folder, `waits on`,
estimate, deadline and due in columns at the right. The folder shows when the rows hold more than
one. A narrow window drops `waits on`, the deadline, the category, the folder, the estimate and the
labels, in that order; the due stays. `<Tab>` folds a header's rows or a parent's children, and
`[[` and `]]` move between headers, in the list and the status window. `<Plug>(dam-today)` opens
`:Dam today`. The winbar names the view, the project and the counts. `!` marks an overdue row and `~` a
blocked one; the `signs` option changes them, such as `signs = { overdue = "x" }`.
A list open in two windows fits its columns to the narrower one.

`:Dam new` opens a one-line prompt that takes `dam new`'s words: `buy oat milk --due fri
--labels errand`. It completes flag names, labels, paths and categories, and after each pause
shows how dam reads the line, or dam's refusal. `<CR>` creates the task, `<C-j>` creates it and
keeps the prompt open, `<Up>` and `<Down>` recall earlier lines, `<Esc>` cancels. `:Dam new
<words>` creates it at once. In a list the prompt opens along the list window's bottom: `a`
with the view's path, `o` with `--parent` set to the task under the cursor.

Keys on list rows act on the visual selection, else the marked rows, else [count] rows, else
the row. Two or more rows go as one dam call after a float shows dam's preview; `y` sends it.
`.` repeats the last key on the rows under the cursor.

| Key | Does |
| --- | --- |
| `x`, `X` | `dam done`; reopen |
| `gx` | `dam done` with the flags you type, such as `--force` |
| `p1` to `p4` | set the priority |
| `cd`, `cD`, `cl`, `cL`, `cg`, `ce`, `cr` | a prompt for `--due`, `--deadline`, `--labels`, `--unlabels`, `--category`, `--estimate` or `--recurrence`, showing each row's before and after |
| `cs` | change the subject of one row |
| `cm`, `cP` | `dam mv` to a `--path` or under a `--parent` |
| `cc` | any `dam edit` flags |
| `dd`, `d` in visual mode | `dam rm`, after the float |
| `U` | `dam restore` to the last commit, after the float |
| `s` | `dam add` |
| `m`, `M` | mark or unmark; unmark all |

A refusal shows as an error on the rows dam names.

`:Dam task <oid>`, or `<CR>` on a row of the list, opens the task as `dam show` prints it: one
`field: value` line each, then `---` and the body. Change a line and `:w` sends `dam edit`
with what changed; an emptied line clears that field. dam reads each value and shows its error
when it refuses one.

`:Dam` or `:Dam status` shows what `dam status` shows in the window's folder: conflicts, waiting
items and sets, unstaged and staged changes, unpushed commits, notices and warnings. Its keys:
`s` stages a change, or everything on the Unstaged or Waiting header; `S` stages everything; `u`
unstages; `a` accepts a waiting item or set and `dd` declines it after a float; `U` sets a change
back to its last commit after a float; `cc` commits; `P` shows what a push would send, then
pushes; `F` pulls; `<CR>` opens the task; `r` reads it again. `:Dam commit [message]`, `:Dam push [remote]` and
`:Dam pull [remote]` do the same from anywhere; `:Dam cancel` stops a slow push or pull. A
conflict is settled with `dam resolve` in a terminal.
A push or pull shows each step dam reports as a Neovim progress message while it runs.
Every open list and status window reads again when Neovim gains focus, so what an agent or a
background sync wrote while you were away is on screen.

`:Dam capture` opens the `dam new` prompt with the line under the cursor as the subject, its
comment leader and any `TODO:` taken off, and `--body "<file>:<line>"` after the cursor. A range
or a selection joins its lines. dam runs in the file's folder, so the task lands in that
project. `:Dam capture!` creates it without the prompt. Map it yourself:

```lua
vim.keymap.set({ "n", "x" }, "<leader>dc", "<Plug>(dam-capture)")
```

`:Dam handoff [note]`, or `H` on a row of a list or the status window, hands the task under the
cursor, or the open task, to an agent: the subject, short oid, path, labels, due, body, your note
and the `dam show` line to read it again. `H` asks for the note. The `handoff` option picks where
it goes: `host = "auto"` sends it to another pane when Neovim runs in Rex, else copies it to the
clipboard; `"rex"` or `"clipboard"` picks one. When Rex refuses it, the brief is copied instead.
The Rex host was written from Rex's help and sends the brief as one paste, then Enter.

`:checkhealth damnit` says whether dam was found and its version fits the plugin, shows each
check of `dam doctor` with dam's advice, and names the hand-off host.

`require("damnit").statusline()` gives short counts for a statusline, such as `!2 ?1 M3 S1 ^2`:
overdue, waiting, unstaged, staged and unpushed commits, using the `signs` characters. It is
empty when all are zero. It never waits on dam: it reads again after a change, when Neovim gains
focus and when the folder changes. `statusline({ highlight = true })` colours each count, and
`require("damnit").counts()` gives the numbers as a table.

```lua
vim.o.statusline = "%f %m%=%{%v:lua.require'damnit'.statusline()%} "
require("lualine").setup({ sections = { lualine_x = { function() return require("damnit").statusline() end } } })
```

With lazy.nvim from a checkout:

```lua
{ dir = "/path/to/damnit/nvim", main = "damnit", opts = {} }
```

From GitHub:

```lua
{
  "webdavis/damnit",
  config = function(plugin)
    vim.opt.rtp:append(plugin.dir .. "/nvim")
    require("damnit").setup({})
  end,
}
```

## Errors

With `-o` a failure prints one document on standard error and nothing on standard
output:

    {"error": {"kind": "refused", "rule": "blocked",
     "message": "98d8780 has 1 open child: a9db854; pass --force to complete it anyway",
     "oids": ["98d878013fb0e026d37170e7ceed6707192ae99a",
              "a9db854060d1943ef9eb9f6d7a8ac0b1ace45d77"]}}

`docs/schemas/error.schema.json` is the JSON Schema of this document.
`kind` is one of `refused`, `store`, `helper`, `credential`, `filter`, `editor`, `parse`,
`usage`, `cancelled`, `scheduler` and `internal`, the last meaning a bug in `dam` itself.
`rule` names which rule a refusal broke, such as `blocked`, `no_such_object` or
`nothing_to_commit`, and is null for every other kind except an ambiguous oid prefix, a `usage`
failure with rule `ambiguous_prefix`. `oids` names the objects the message names, in the order it
names them.

`dam doctor`, `dam test` and `dam config validate` are the exception: their report goes to
standard output, with exit 1 when a check finds an issue, and no error document.

Standard error carries that one document and nothing else, so parse the whole stream; a run that
succeeds writes nothing there, unless a hook prints or warns. A `hook_refused` error carries
`hook`, the event whose hook refused. A mistyped command line also carries `usage`, which says what was
wrong with it. An error about an id you gave, such as `no_such_object`, also carries `prefix`, that
id as the message shows it: the text you gave, or the first seven characters of a whole id. Without
those flags you get the plain `dam: <message>` line, or clap's usage text for a command line it
rejects.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | The command did what it was asked. |
| 1 | `dam` failed: a store, config, helper, editor or io failure, or a bug in `dam` itself. |
| 2 | The command line was wrong: an unknown argument, an unreadable value, or a bad oid. |
| 3 | Cancelled: you interrupted, a prompt could not be answered, or you saved an edit untouched. |
| 4 | `dam` refused by one of its own rules, and the message names the rule. |
| 5 | `dam` refused the token in `DAM_TOKEN`, and the message names the rule. |

Every rule `dam` keeps reports code 4, except a refused token, which reports code 5. A `--recurrence` rule `dam` cannot
read is the one flag value outside code 2: it exits 1, and its error document's kind is `parse`.
