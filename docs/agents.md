# Sending tasks from a tool or an agent

A tool or an agent runs `dam` with a token. This page lists what to pass and what comes back.

## The token

Pass the token in the `DAM_TOKEN` environment variable. No flag takes it.

The owner makes the token and prints it once. A tool that only sends tasks needs `task:create`:

```sh example
dam token create tool --allow task:create --paths inbox --require session
```

An agent that manages its own tasks also needs `task:read`, `task:change`, `task:complete` and
`task:remove`, and `event:read` for `dam agenda`.
`--own` narrows the grant to what the token made: tasks it sent that the owner added, and its own
waiting items. It never stops a `dam new`.

```sh example
dam token create agent --allow task:read --allow task:create --allow task:change --allow task:complete --allow task:remove --allow event:read --paths work --own
```

`--paths` limits the token to objects under that path, so each task the token sends needs a
`--path` under it. The README's Tokens section lists every permission and grant key.

The project comes from the folder `dam` runs in, so a tool or agent that runs `dam` inside a
project folder gets that project, the same as a person does. A tool that wants every task passes
`--global`. The nearest `.dam.lua` at or above the folder is the project, and a project inside
another uses only its own file.

In a folder that holds `.dam.lua`, `dam new --path` starts from the project's path, and the
token is checked on the full path; `--global`, or `"global": true` in a request, starts it from
the root, and `--project <path>`, or `"project"` in a request, starts it from that path from any
folder. A missing task for that path is created first; that needs `task:create` there, and the
answer carries `project_created`. The path `dam mv` moves objects to starts from the project the
same way; its oids, `--query` and `--filter` still pick from every task. `dam ls`, `dam agenda` and `dam reminders` there list only what the token sees under the
project. `dam config list --json` carries every setting the project's `.dam.lua` holds under
`project`, as the file sets it.

When a template renders `secrets.lua`, the template writes the token's table with the value
`dam token create` printed.

## Sending a task

```sh example as=agent
dam new "Draft the release notes" --path work/ --due fri -p 2 --labels writing
```

The same request as one JSON document, on standard input or in a file:

```sh example as=agent
dam new --request - <<< '{"subject": "Draft the changelog", "path": "work/", "due": "fri", "labels": ["writing"]}'
printf '%s' '{"subject": "Tag the release", "path": "work/"}' > task.json
dam new --request task.json
```

The document's keys are the flags without their dashes, plus `subject`. Only global flags, such as
`--json`, may come beside `--request`. The schema is `docs/schemas/new.request.schema.json`;
`edit.request.schema.json`, `done.request.schema.json` and `rm.request.schema.json` cover the other
writing commands.

`--template <name>` (request key `template`) starts the task from the project's
`.damtemplates/<name>.lua`, else `templates/<name>.lua` beside `config.lua`. Each other flag replaces
the template's value. The token's permissions are checked on the task the template makes.

`--parent <oid>` files the task in that object, at its folder or at a `--path` inside it; a path
outside it is refused, rule `path_outside_parent`. When two objects of the same name hold the
`--path`, `dam new` is refused until `--parent` names one: rule `several_hold_the_path` when the
token sees both, else `not_permitted`.

## Your own id

`--id` names the task with your own id. The id is one line of up to 200 characters. A repeat of
`dam new` with the same id changes that task, and only the fields that differ from your last send
change:

```sh example as=agent
dam new "Review the API docs" --path work/ --id api-docs
dam new "Review the API docs" --path work/ --id api-docs -p 1
```

The task is the token's name plus the id, plus the session when the request passes
`--meta session=...`. Two tokens, or two sessions, with the same id make two tasks. A repeat cannot
change `--path`.

## Request details

`--meta key=value` states a detail of the request. Repeat it for each detail: `harness`, `session`,
`model`, `effort`, `version`, or any other key. dam keeps each value as given with the change.

```sh example as=agent
dam new "Update the install guide" --path work/ --meta session=7f3c --meta model=example-model --meta version=1.4.0
```

A request document carries them as `meta`, an object of strings:
`{"subject": "Update the install guide", "meta": {"session": "7f3c"}}`. Pass `meta` in the document
or `--meta` on the line, not both.

A token's `require` names details every request must pass. A request without one exits 5, rule
`missing_details`, and the message names the missing `--meta`:

```sh example as=tool exit=5 rule=missing_details
dam new "Renew the passport" --path inbox/ --json
```

```sh example as=tool
dam new "Renew the passport" --path inbox/ --meta session=7f3c
```

## What comes back

```sh example as=agent
dam new "Fix the flaky login test" --path work/ --id flaky-login --json
```

```json
{"answer": "new", "hash": "a1b2c3d4...", "oid": "a1b2c3d4...", "item": "new", "kind": "task",
 "subject": "Fix the flaky login test", "path": "work/", "from": "agent",
 "key_id": "k3y1d0000000", "channel": "local", "created": "2026-11-12T09:00:00Z",
 "id": "flaky-login"}
```

`answer` is `new`, `changed` or `duplicate`. `duplicate` stores nothing and names the item that
already says the same. `hash` names the item. `oid` is the task it makes or changes; for a new item
it equals `hash`. `--toon` gives the same document in TOON. The schema is
`docs/schemas/sent.schema.json`.

A token's write never touches the owner's tasks. It waits as an item until the owner adds it.
Meanwhile `dam status --json` lists the token's own items under `waiting`, `dam ls` leaves the
task out, and `dam show` refuses the item's hash with rule `item_not_an_object`:

```sh example as=agent exit=4 rule=item_not_an_object
dam show <hash> --json
```

```sh example as=agent
dam status --json
```

The owner takes the item:

```sh example
dam add <hash>
dam commit -m "take the agent's task"
```

## Changing and finishing your task

`dam edit` and `dam done` on a task the owner has added send a change item, answered the same way.
Before the owner adds the task, repeat `dam new` with its `--id` instead.

```sh example as=agent
dam edit <oid> --due mon --json
```

`dam rm <hash>` withdraws an item of yours that is still waiting:

```sh example as=agent
dam rm <hash>
dam done <oid>
```

`dam rm <oid>` on a task the owner has added sends a removal item. It needs `task:remove`. The task
stays until the owner adds or declines the removal. Both answer as the next section shows.

A token cannot run `dam mv`, `dam done --force`, or `dam edit --focus`. Each exits 4, rule
`not_open_to_tokens`. A project's `done_children` changes only what `dam done --force` does with
open children, never what a token may do.

`dam edit`, `dam done` and `dam rm` also take several oids, `--query TEXT` or `--filter NAME`. The
set holds only the objects your grants let you see, and each change waits as its own item. Add
`--preview` to see the changes without sending anything, then run again with `--expect N` so the run
refuses if the set changed since. A request carries `oid` as a string or an array, or `query` or
`filter`, beside `preview` and `expect`. The answer is `changes.schema.json`: `count` and `oids` are
the set, and each change in `changes` carries its `item` and `answer`. `dam new --preview` answers
the same document for the item it would send, and sends nothing.

Add `--as-one` to make the whole set wait as one set of items instead. The answer's `set` is its
hash; the owner takes or declines the set whole, by that hash. `--as-one` from the owner exits 2.
`dam add <set hash>` takes every item in the set or none: if one no longer applies, nothing is
taken. `dam add -A` takes each set whole or leaves it waiting. `dam rm <set hash>` declines the set,
or withdraws it when its token runs it. `dam resolve <set hash> --theirs` takes the set whole and
`--ours` declines it. An item in a set is never taken or declined alone; the refusal names the set.
`dam status` shows a waiting set as one row with its count, and `dam show <set hash>` lists its
changes. A token finds a set only when it sent it or can see every item in it.

`dam category migrate` is the owner's alone; a token is refused, rule `not_open_to_tokens`. Its
request is `{"map": {"quick": "light"}, "preview": true, "expect": 3, "update_config": true}`
(`category-migrate-request.schema.json`), and its answer is `category-migrate.schema.json`: the
changes document plus `config`, each place in `config.lua` that names an old label, with `written`.

If your token may push, `dam push --preview [remote]` shows what a push would send, hold and skip,
listing only the objects you may read, and contacts nothing. In a data format it answers
`push-preview.schema.json`.

`dam push` names and counts only the objects you may read: `sent`, `succeeded`, `skipped` and
`held` leave the others out, and `failed` lists only yours. A remote's own
words for a refusal reach you only when your token reads every task and event; otherwise the reason
reads `refused by the remote; ask the owner to run dam status for the reason`. An object counts as
yours only when you may read both its working copy and its last committed one. When a change you cannot see failed or was held, the answer
says `some changes you cannot see did not go through; ask the owner to run dam status`, and in a
data format carries `"unseen_failed": true`. `dam sync run` answers the same way.

`dam status` shows a conflict, a notice or an unpushed change only for an object you may read,
and a push failure's reason only when your token reads every task and event. When a conflict,
failed push or unpushed change you cannot see is there, it ends with the same sentence and carries
`"unseen_failed": true`.

## Taking your task back

`dam rm --id <id>` takes back the task you sent with `dam new --id <id>`. Pass the same
`--meta session=...` you passed to `dam new`; another session's id names another task.

```sh example as=agent
dam rm --id flaky-login --json
```

```json
{"answer": "waiting", "item": "e4f5a6b7...", "oid": "a1b2c3d4..."}
```

| `answer` | What happened | `item`, `oid` |
|---|---|---|
| `withdrawn` | Your item was still waiting. It is gone. | The item, the task. For a new task they are the same hash. |
| `waiting` | The owner had added the task. A removal item waits for the owner to add or decline. A repeat names the same item. | The removal item, the task |
| `declined` | The owner declined that removal. The task stays, and the removal is not sent again unless the owner puts it back to waiting with `dam reset <hash>`. | The removal item, the task |
| `nothing` | Nothing of yours is under that id: never sent, already withdrawn, removed, outside your grants, or your item was declined before the owner added it. | Null, null |

Every answer exits 0, so a repeat is safe:

```sh example as=agent
dam rm --id never-sent
```

Sending a removal needs `task:remove`, or `event:remove` for an event. Without it, `dam rm` exits
4, rule `not_permitted`. Withdrawing a waiting item needs only the token.

`dam rm <hash>` and `dam rm <oid>` give the same answers, with one difference: an oid or prefix
outside your grants exits 4, rule `no_such_object`, as a missing one does. Text mode prints one
line, such as `waiting  e4f5a6b  a1b2c3d`, or `withdrawn  a1b2c3d` when the item and the task are
the same hash. In a request document the key is `id`. The schema is
`docs/schemas/removal.schema.json`.

## Reading

```sh example as=agent
dam ls --json
dam ls 'due:today | overdue' --json
dam ls --by agents --json
dam show <oid> --json
dam agenda --json
dam reminders --json
```

`dam ls --by mine`, `--by agents` and `--by all` list what the owner made, what tokens made, or
both. A grant with `--own` lists only events the token made. Each `dam agenda --json` row carries
the event's `attendees`, and its `organizer`, `location` and `conference` when it has them. An
object outside the token's grants answers like a missing one: left out of lists, and exit 4, rule
`no_such_object`, when named.

`dam ls`, `dam show`, `dam agenda` and `dam reminders` answer from the local store and start no
remote helper; `dam pull` brings a remote's changes in. `--max-age <remote>=<duration>` makes
`dam agenda` refuse, exit 4, rule `stale_remote`, when that remote last pulled longer ago,
here a remote named `calendar`. A token needs `remote:read` on that remote, or it gets rule
`no_such_remote`:

```sh example exit=4 rule=stale_remote
dam agenda --json --max-age calendar=15m
```

## Exit codes and errors

| Code | Meaning | Retry |
|---|---|---|
| 0 | Done. | |
| 1 | A store, config, helper, editor or io failure, or a bug in `dam`. | Once |
| 2 | The command line was wrong. | No |
| 3 | Cancelled: interrupted, or a prompt could not be answered. | After an interrupt |
| 4 | Refused by a rule; the error names it. | Only `stale_remote`, once the remote has pulled |
| 5 | The token was refused; the error names the rule. | No |

With `--json` or `--toon`, a failure prints one document on standard error and nothing on standard
output:

```json
{"error": {"kind": "refused", "rule": "no_such_object",
 "message": "no object matches \"a1b2c3d4\"", "oids": [], "prefix": "a1b2c3d4"}}
```

`rule` names the rule a refusal broke, and is null for other failures. `prefix` is the id the
error is about, as the message shows it, on `ambiguous_prefix`, `no_such_object`,
`no_working_object` and an id that is not an oid: the text you gave, or the first seven characters
of a whole id. A send with `--id` is safe to
retry: a repeat answers `duplicate`. The schema is `docs/schemas/error.schema.json`, and it lists
every rule.

A hook in the `hooks` folder beside `config.lua` runs on the owner's `dam new`, `dam edit`,
`dam done` and `dam add`.
A token's command runs no hook; its item runs one when the owner's `dam add` takes it.
A token cannot run `dam hook list` or `dam hook run`, which are the owner's alone.
A project's hooks in `.damhooks/` run only once the owner turns them on with `dam config set
project.hooks true`, which no token can run; then they run after the owner's, for tasks under the
project's path. A token's command runs no hook at all, the owner's or a project's.
Inside a project, the owner's `dam config set` and `unset`, `dam filter create|remove` and
`dam category create|remove` write `.dam.lua`, and `--global` writes `config.lua`; in a data format the answer's `file` names the file written,
`.dam.lua` relative to the project's folder. No token can run it.
A hook may return a `dam edit --request` document for its task, which the same command applies.
The answer of `new` and `edit` then shows the task as the hook left it. The answer of `new`,
`edit`, `done` and `add` carries `changed_by_hook`: each task a hook changed, as `oid` and `hook`.
It is left out when no hook changed anything.
What it prints and its warnings go to standard error. A hook set to refuse exits 4, rule
`hook_refused`, and the error names the event under `hook`.
A hook's own commands run with `DAM_HOOK` set to the event. A `dam` that finds `DAM_HOOK` set
exits 4, rule `started_by_hook`, before it reads or writes anything.
A pull runs only `task_pulled`, once for each task it created or changed, after the pull
has landed. A failing one never changes the pull's exit code. The answer of `pull` carries
`task_pulled` on each remote it ran for: the counts `ran`, `changed`, `failed` and `not_reached`,
and `commit`, the id of the commit that holds the hook's changes when it made one.
`not_reached` counts the tasks the hook was not called for once the pull's 60 seconds for it ran
out.
An interrupt during the hook keeps the pull and the hook's changes, and exits 3, `cancelled`.
The line `dam: running task_pulled for <n> tasks` shows on standard error only in plain text.
A run that failed on a task or did not reach one leaves one notice per remote in `dam status`,
kind `task_pulled_failed`, with `remote`, `failed`, `not_reached` and `error`, the first error's
`file`, `line` and `message`, or null. The next run of that remote replaces or clears it.
A token's `dam status` never shows it.

A mistyped command line also carries `usage`, which says what was wrong with it:

```json
{"error": {"kind": "usage", "rule": null, "message": "unexpected argument '--bogus' found",
 "oids": [], "usage": {"problem": "unknown_flag", "word": "--bogus"}}}
```

`problem` is `unknown_subcommand`, `unknown_flag`, `unexpected_argument`, `missing_value`,
`bad_value`, `missing_argument`, `conflict` or `other`. `word` is the subcommand or flag at fault, or
null. A `dam` older than your tool answers `unknown_flag` or `unknown_subcommand` with the word your
tool passed. A `dam` from before this document answers exit 2 with plain text and no document.

## Schemas

Every schema is in `docs/schemas/`. A command's answer is `<command>.schema.json`, such as
`token-create.schema.json` for `dam token create`, and its request is
`<command>.request.schema.json`. `object.schema.json` is a task or an event.
