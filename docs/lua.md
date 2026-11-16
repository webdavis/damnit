# Lua reference

dam reads two Lua files from one folder: `config.lua` and `secrets.lua`. Each returns a table. A
key dam does not know is an error. A `<name>` in a key is a name you choose.
Hooks live in their own files in a `hooks` folder beside them.

## config.lua

| Key | Value |
| --- | --- |
| `owner` | The name your own objects and changes carry as their author. Default: `owner`. |
| `done` | A table. |
| `done.interactive` | `true` makes `dam done` ask about open children and dependencies. |
| `progress` | `false` hides progress lines unless you give `--progress`. Default: `true`. |
| `remote` | A table of remotes by name. |
| `remote.<name>.url` | `<helper>::<address>`. Required. |
| `remote.<name>.deadline` | A duration. The longest the helper may stay silent. Default: `60s`. |
| `remote.<name>.sync` | A table. |
| `remote.<name>.sync.interval` | A duration of at least `1m`. How often background sync runs. Default: `15m`. |
| `remote.<name>.sync.allowed` | `false` means background sync may never start for this remote. Default: `true`. |
| `remote.<name>.path` | A push sends only the objects under this path. |
| `category` | A table of label categories by name. |
| `category.<name>.values` | A list of labels. Required. |
| `category.<name>.exclusive` | `true` allows one value per object. |
| `filter` | A table of saved filters by name. |
| `filter.<name>.query` | Query text. |
| `filter.<name>.match` | A function that returns `true` to keep an object. |

A duration is a whole number followed by `s`, `m` or `h`.

`dam config set <key> <value>` and `dam config unset <key>` change one key of this table.

## Hook files

Each hook is one file in the `hooks` folder beside `config.lua`, named for its event.
A hook file returns the hook: a function, or a table that holds the function as `run`.
Only files ending in `.lua` count. A `.lua` file named for no event is an error.
A `hook` table in `config.lua` is an error that names the file each hook moves to.
A hook file holds no secret, so others may read it.

| Event | Hook |
| --- | --- |
| `task_new` | `hooks/task_new.lua` runs when you create a task. |
| `task_changed` | `hooks/task_changed.lua` runs when you change a task. |
| `task_done` | `hooks/task_done.lua` runs when you complete a task. |
| `task_pulled` | `hooks/task_pulled.lua` runs for each task a pull brings in. It takes no `on_failure`. |
| `<event>.run` | The hook's function, when the hook is a table. Required. |
| `<event>.on_failure` | `warn` or `refuse`. Default: `warn`. |
| `task_pulled.changes` | `committed` or `working`: where its changes go. Default: `committed`. |

## secrets.lua

| Key | Value |
| --- | --- |
| `schema` | `1`. Another value disables every token. |
| `remote` | Credentials by remote. Each remote must be declared in `config.lua`. |
| `remote.<name>.<credential>` | A string, or a function that returns one. dam calls it only when the remote is used. |
| `token` | Tokens by name. `dam token create` writes them. |
| `token.<name>.value` | The token. Required. |
| `token.<name>.expires` | An RFC 3339 time after which the token is refused. |
| `token.<name>.not_before` | An RFC 3339 time before which the token is refused. |
| `token.<name>.via` | A list of `local`, `mcp` or both. Default: `local`. |
| `token.<name>.require` | Details each request must state with `--meta KEY=VALUE`. |
| `token.<name>.limits` | Must be empty. Any limit disables the token. |
| `token.<name>.grant` | A list of grants. |
| `token.<name>.grant[].allow` | The permissions the grant gives. Required. |
| `token.<name>.grant[].paths` | Only objects under one of these paths. |
| `token.<name>.grant[].labels` | Only objects that carry one of these labels. |
| `token.<name>.grant[].own` | `true` gives only objects a token of this name made. |
| `token.<name>.grant[].remotes` | Only these remotes. |

## Filters

A filter takes `query`, `match`, or both. A filter with neither is an error. `dam ls <name>` and
`dam agenda <name>` run it. With both, the query narrows first, and `match` runs only on the
objects that are left.

`match(task, dam)` runs once for each object the listing would show, events as well as tasks.
`task.kind` says which. It returns `true` to keep the object. `false` and `nil` drop it, as Lua
treats `nil` as false. Any other value is an error.

`task` is a copy, so writing to it changes nothing. `dam.history(task)` returns that object's
changes. A filter function cannot read other objects or change anything.

`dam test filters` runs every saved filter once and names the first object a `match` fails on.

A token caller has `match` run only over the objects it may read. For a token without
`history:read` on everything, `dam.history` raises an error.

## What match receives

`task` has the fields `dam show --json` prints, except `author`, `details` and `id`. Those three
come from dam's records of who made the object and the id a caller gave it, not from the object
itself. `dam.history(task)` gives the `author` and `details` of each change. A filter cannot read
`id`. An optional field is `nil` when it holds nothing.

| Field | Value |
| --- | --- |
| `oid` | The object's id. |
| `kind` | `task` or `event`. |
| `subject` | The title. |
| `body` | The notes. |
| `path` | The folder it sits under, such as `work/`, or empty. |
| `labels` | A list of labels. |
| `depends` | A list of the oids it waits on. |
| `reminders` | A list of minutes before the due time or start. |
| `recurrence` | The rule it repeats by. |
| `parent` | The oid of the object it sits in, or nil. |
| `task.done` | `true` or `false`. |
| `task.completed_at` | When it was completed. |
| `task.priority` | `1`, the highest, to `4`. |
| `task.due` | A date, or a date-time with its offset. |
| `task.deadline` | A date. |
| `task.attached` | The oid of the event it is attached to. |
| `task.estimate` | Minutes it should take. |
| `task.focus.day` | The day it is marked to work on first. |
| `task.focus.set_by` | `owner`, or the token that marked it. |
| `task.focus.set_at` | When it was marked. |
| `event.start` | A date, or a date-time with its offset. |
| `event.end` | A date, or a date-time with its offset. |
| `event.timezone` | The time zone name. |
| `event.location` | Where it is. |
| `event.attendees[].email` | An attendee's email. |
| `event.attendees[].response` | `needs_action`, `accepted`, `declined` or `tentative`. |
| `event.attendees[].self` | `true` for you. |
| `event.status` | `confirmed`, `tentative` or `cancelled`. |
| `event.transparency` | `busy` or `free`. |
| `event.visibility` | `default`, `public`, `private` or `confidential`. |
| `event.event_type` | `default`, `focus_time`, `out_of_office`, `working_location` or `birthday`. |
| `event.color` | The color. |
| `event.organizer.email` | The organizer's email. |
| `event.organizer.name` | The organizer's name. |
| `event.conference.provider` | The call's provider. |
| `event.conference.url` | The call's link. |
| `event.attachments[].url` | An attachment's link. |
| `event.attachments[].title` | An attachment's title. |
| `event.attachments[].mime_type` | An attachment's type. |

## What dam.history returns

A list of commits, newest first, as `dam log --json` prints them. Each commit holds only this
object's changes. A change shows the object as of that change. `changes[].details` is what the
request stated, and `nil` when it is not shown to the caller.

| Field | Value |
| --- | --- |
| `id` | The commit's id. |
| `at` | When it was committed. |
| `message` | The commit message. |
| `changes[].oid` | The object's id. |
| `changes[].op` | `create`, `update` or `delete`. |
| `changes[].fields` | The names of the fields the change set. |
| `changes[].kind` | `task` or `event`. |
| `changes[].subject` | The title. |
| `changes[].path` | The folder. |
| `changes[].labels` | The labels. |
| `changes[].done` | For a task, `true` or `false`. |
| `changes[].completed_at` | For a task, when it was completed. |
| `changes[].priority` | For a task, its priority. |
| `changes[].due` | For a task, when it was due. |
| `changes[].start` | For an event, its start. |
| `changes[].end` | For an event, its end. |
| `changes[].from` | The token that made the change, or `nil` for you. |
| `changes[].key_id` | That token's key id. |
| `changes[].channel` | That token's channel. |
| `changes[].author` | The token's name, or `owner` from `config.lua`. |
| `changes[].details.meta` | The `--meta` pairs. |
| `changes[].details.host` | The host. |
| `changes[].details.dir` | The working directory. |
| `changes[].details.repo` | The repository. |
| `changes[].details.branch` | The branch. |
| `changes[].details.at` | When the request was made. |
| `changes[].details.channel` | `local` or `mcp`. |

## Hooks

A hook is a function dam calls when your own command changes a task.
`task_new` runs for a task your `dam new` creates.
`task_done` runs for a task your `dam done` completes, or rolls forward when it repeats.
`task_changed` runs for any other change your `dam new`, `dam edit` or `dam done` makes.
A token's item runs the same hooks when your `dam add` or `dam resolve <hash> --theirs` takes it.
A command runs one hook for each task it changed, so `dam done --force` that moves open children
runs `task_changed` for each child.
When the item removes a task, the removed task runs no hook, and each task the removal moves or
frees runs `task_changed`.
Events, `dam rm`, `dam restore`, `dam resolve --ours`, `dam resolve` of a conflict with a remote,
`dam revert`, `dam reset`, a pull and a token's commands run none of these three. A pull runs only
`task_pulled`.

A hook file returns a function, or a table that holds the function as `run`. `run(task, dam)` gets what a
filter's `match` gets, with the task as it is after the change.
It returns nothing, or a table of fields to change on that task.
The table takes the keys and values of a `dam edit --request` document, except `oid` and `meta`.
dam applies it to the task within the same command, by the rules `dam edit` applies.
The change a hook makes runs no hook.
The command's answer lists each task a hook changed under `changed_by_hook`, as its `oid` and
the `hook`'s event. In plain text it says `dam: hook task_new changed <oid>` on standard error.
`dam log` names the hook beside the sender, as `you, hook task_new`, and `--json` holds it
under `details.meta.hook`. When several hooks change a task before your `dam add`, the record
names the last one. When your own edit and a hook's edit land in one staged change, the record
says a hook changed it, not which fields the hook set. `dam restore` or `dam rm` of the task
drops what a hook did from that record; `dam reset` keeps it.
A command a hook starts with `os.execute` or `io.popen` runs with `DAM_HOOK` set to the event.
A `dam` that finds `DAM_HOOK` set refuses at once, before it reads or writes anything.
It exits 4 with rule `started_by_hook`.
A program started another way, such as through a Lua library other than `os` and `io`, does not
carry `DAM_HOOK`, and neither does one that clears its environment.
A `dam` it starts waits on the busy store and fails, and the hook runs over.

A hook fails when it raises an error, returns a value other than nil or a table, or runs over.
It also fails when its table is not an edit request, or when `dam edit` would refuse that edit.
Then dam applies none of the hook's table.
With `on_failure = "warn"`, the default, the change stands and dam warns on standard error:

    warning: hook task_done failed at /home/ada/.config/dam/hooks/task_done.lua:3: no bell; the change stands

With `on_failure = "refuse"`, the command changes nothing and exits 4 with rule `hook_refused`.
Its error document names the event under `hook`.

A failing hook's warning says what was wrong with its table:

    warning: hook task_new failed at /home/ada/.config/dam/hooks/task_new.lua:2: its edit request failed: --request: colour is not a field it takes; the change stands

`task_pulled` runs after a pull has landed, once for each task the pull created or changed.
`run(task, before, pull)` gets the task as the pull left it, and `before`, dam's task before the
pull, which is nil for a new task. Both have the shape `match` gets.
`pull` holds `remote`, the remote's name, and `change`: `new`, `changed`, or `done` when the task
went from open to done.
It does not run for a task the pull found unchanged, held as a conflict, left alone for your
newer work, or removed, nor for an event.
It does not run on the first pull from a remote, so the tasks of that pull never get it.
It may return a table of fields, as the other hooks do.
With `changes = "committed"`, the default, dam commits those changes in one commit of their own
right after the pull's commit: `task_pulled after pull from <remote>`, by you, each change's
details holding `hook = task_pulled` in `meta`. The commit is owed to the remote like any other;
the pull pushes nothing. No change, no commit.
With `changes = "working"`, dam leaves them as working changes, as the other hooks do.
When it fails on a task, dam drops that task's change, goes on to the next task, and keeps the
pull. The pull's exit code stays as it was.
The first failure of a pull warns on standard error, unless a token's command caused the pull:

    warning: hook task_pulled failed at /home/ada/.config/dam/hooks/task_pulled.lua:3: no bell; the pull is kept

Later failures of that pull are counted, not printed.
After 60 seconds of one pull's run, dam calls the hook for no more tasks and counts them in
`not_reached`.
Ctrl-C stops the hook and keeps the pull and the changes the hook made; the command exits 3.
While the run goes, a terminal shows its progress on standard error, unless a token's command
caused the pull. The line stays when the run ends:

    ✓ ran task_pulled  40 of 40 tasks  3s

When a pull's run failed on a task or did not reach one, `dam status` keeps one notice for that
remote, also when a token's command caused the pull:

    task_pulled on the last pull from todoist: 2 failed, 0 not reached; first error at /home/ada/.config/dam/hooks/task_pulled.lua:3: no bell

The next run of that remote replaces it, or clears it when it fails on no task and reaches all.
A token never sees it.
The pull's answer counts what it did under `task_pulled`: `ran`, `changed`, `failed` and
`not_reached`, and `commit`, the id of the commit it made. In plain text it is a line such as
`task_pulled: 1 ran, 1 changed, 0 failed, 0 not reached, commit 2b2b2b2`.
When a remote rewrites what dam pushed, the task comes back changed and the hook runs on it once
more. A hook that changes a task each time it runs changes it once per push and pull, never in a
loop.

Tasks you complete, written to a file, in `hooks/task_done.lua`:

    return function(task)
      local log = io.open(os.getenv("HOME") .. "/done.log", "a")
      log:write(task.subject, "\n")
      log:close()
    end

No task left outside a folder, in `hooks/task_new.lua`:

    return {
      run = function(task)
        if task.path == "" then
          error("file it under a folder")
        end
      end,
      on_failure = "refuse",
    }

A due date on each changed task that has none, in `hooks/task_changed.lua`:

    return function(task)
      if task.task.due == nil then
        return { due = "tomorrow" }
      end
    end

## Limits and errors

One filter has 5 seconds in total for a whole listing. One hook call has 5 seconds.
`task_pulled` has 60 seconds in total for one pull.
dam checks the time while Lua code runs.
The limit covers code in a coroutine, and a `pcall` or `xpcall` cannot catch it for long.
It cannot stop a blocked external command, such as one started with `os.execute`. It notices the
overrun only when that command returns.
It cannot stop one long call into Lua's library, such as `string.find` with a pattern that
backtracks for a long time.
It cannot stop a `__gc` method, an `xpcall` handler that runs as dam stops the code, or a
`__close` method that runs as dam stops the code.

What `match` or a hook prints, with `print` or `io.write`, goes to standard error.

A filter that raises an error, runs over, or returns another value fails the command with exit 1.
For you, the message names the filter, the file and the line, then Lua's own message:

    dam: filter broken failed at /home/ada/.config/dam/config.lua:10: attempt to index a nil value (field 'due')

A token caller sees only the filter's name:

    dam: filter broken failed; the owner sees why by running the same command

`dam filter list` shows each filter's kind, `query` or `function`, and never a function's source.
`dam filter create` writes only `query`, and `dam filter remove` refuses a filter with `match`;
edit those in `config.lua` by hand.

## Example

Tasks whose due date moved more than twice:

```lua
return {
  filter = {
    postponed = {
      match = function(task, dam)
        if task.kind ~= "task" then
          return false
        end
        local moves = 0
        for _, commit in ipairs(dam.history(task)) do
          for _, change in ipairs(commit.changes) do
            for _, field in ipairs(change.fields) do
              if change.op == "update" and field == "due" then
                moves = moves + 1
              end
            end
          end
        end
        return moves > 2
      end,
    },
  },
}
```
