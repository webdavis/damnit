# Sending tasks from a tool or an agent

A tool or an agent runs `dam` with a token. This page lists what to pass and what comes back.

## The token

Pass the token in the `DAM_TOKEN` environment variable. No flag takes it.

The owner makes the token and prints it once. A tool that only sends tasks needs `task:create`:

```sh example
dam token create tool --allow task:create --paths inbox --require session
```

An agent that manages its own tasks also needs `task:read`, `task:change` and `task:complete`,
and `event:read` for `dam agenda`.
`--own` narrows the grant to what the token made: tasks it sent that the owner added, and its own
waiting items. It never stops a `dam new`.

```sh example
dam token create agent --allow task:read --allow task:create --allow task:change --allow task:complete --allow event:read --paths work --own
```

`--paths` limits the token to objects under that path, so each task the token sends needs a
`--path` under it. The README's Tokens section lists every permission and grant key.
When a template renders the config, the template writes the token's table with the value
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

A token cannot run `dam mv`, `dam rm` on a task, `dam done --force`, or `dam edit --focus`. Each
exits 4, rule `not_open_to_tokens`.

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

On `dam ls`, `dam show`, `dam agenda` and `dam reminders`, `--no-pull` answers from the local
store and starts no remote helper; every other command refuses it with exit 2. `--max-age <remote>=<duration>`
makes `dam agenda` refuse, exit 4, rule `stale_remote`, when that remote last pulled longer ago,
here a remote named `calendar`. A token needs `remote:read` on that remote, or it gets rule
`no_such_remote`:

```sh example exit=4 rule=stale_remote
dam agenda --json --no-pull --max-age calendar=15m
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
 "message": "no object matches \"a1b2c3d4\"", "oids": []}}
```

`rule` names the rule a refusal broke, and is null for other failures. A send with `--id` is safe to
retry: a repeat answers `duplicate`. The schema is `docs/schemas/error.schema.json`, and it lists
every rule.

A `usage` error also carries `usage`, which says what was wrong with the command line:

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
