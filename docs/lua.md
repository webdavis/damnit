# Lua reference

dam reads two Lua files from one folder: `config.lua` and `secrets.lua`. Each returns a table. A
key dam does not know is an error. A `<name>` in a key is a name you choose.

## config.lua

| Key | Value |
| --- | --- |
| `owner` | The name your own objects and changes carry as their author. Default: `owner`. |
| `done` | A table. |
| `done.interactive` | `true` makes `dam done` ask about open children and dependencies. |
| `remote` | A table of remotes by name. |
| `remote.<name>.url` | `<helper>::<address>`. Required. |
| `remote.<name>.stale` | A duration, such as `15m`. A read pulls first when the last pull is older. |
| `remote.<name>.deadline` | A duration. The longest the helper may stay silent. Default: `60s`. |
| `remote.<name>.path` | A push sends only the objects under this path. |
| `category` | A table of label categories by name. |
| `category.<name>.values` | A list of labels. Required. |
| `category.<name>.exclusive` | `true` allows one value per object. |
| `filter` | A table of saved filters by name. |
| `filter.<name>.query` | Query text. |
| `filter.<name>.match` | A function that returns `true` to keep an object. |
| `hook` | A table of hooks by event. |
| `hook.task_new` | A hook that runs when you create a task. |
| `hook.task_changed` | A hook that runs when you change a task. |
| `hook.task_done` | A hook that runs when you complete a task. |
| `hook.<event>.run` | The hook's function, when the hook is a table. Required. |
| `hook.<event>.on_failure` | `warn` or `refuse`. Default: `warn`. |

A duration is a whole number followed by `s`, `m` or `h`.

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
`hook.task_new` runs for a task your `dam new` creates.
`hook.task_done` runs for a task your `dam done` completes, or rolls forward when it repeats.
`hook.task_changed` runs for any other change your `dam new`, `dam edit` or `dam done` makes.
A command runs one hook for each task it changed, so `dam done --force` that moves open children
runs `task_changed` for each child.
Events, `dam rm`, `dam restore`, `dam resolve`, `dam revert`, `dam reset`, a pull and a token's
commands run no hook.

A hook is a function, or a table that holds the function as `run`. `run(task, dam)` gets what a
filter's `match` gets, with the task as it is after the change. It returns nothing.
A `dam` that a hook starts cannot change anything: the store stays busy until the hook returns.

A hook fails when it raises an error, returns a value, or runs over.
With `on_failure = "warn"`, the default, the change stands and dam warns on standard error:

    warning: hook task_done failed at /home/ada/.config/dam/config.lua:3: no bell; the change stands

With `on_failure = "refuse"`, the command changes nothing and exits 4 with rule `hook_refused`.
Its error document names the event under `hook`.

Tasks you complete, written to a file, and no task left outside a folder:

    return {
      hook = {
        task_done = function(task)
          local log = io.open(os.getenv("HOME") .. "/done.log", "a")
          log:write(task.subject, "\n")
          log:close()
        end,
        task_new = {
          run = function(task)
            if task.path == "" then
              error("file it under a folder")
            end
          end,
          on_failure = "refuse",
        },
      },
    }

## Limits and errors

One filter has 5 seconds in total for a whole listing. One hook call has 5 seconds.
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
