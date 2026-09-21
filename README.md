# damnit

`dam` keeps your tasks and calendar events in a local store and treats them the way git treats
files: a working layer you edit, a stage you `add` to, commits you `push` to a remote, and
`pull` to bring the remote's changes in. Todoist is the first remote.

## Install

    cargo install --git https://github.com/webdavis/damnit damnit
    cargo install --git https://github.com/webdavis/damnit dam-remote-todoist

Both land in `~/.cargo/bin`. `dam` finds a remote helper the way git does: a `dam-remote-<name>`
binary on `PATH`.

## First run

    dam remote add todoist todoist::

Then give the helper a token, in `~/.config/dam/config.toml` under the table `dam remote add`
wrote:

    [remote.todoist]
    url = "todoist::"
    api_token_command = ["security", "find-generic-password", "-w", "-s", "Todoist API Token"]
    stale = "15m"

`api_token` (the value, with `credentials = ["api_token"]` in the same table), `api_token_command`
(a command that prints it) and `api_token_env` (a variable name) are the three forms; the command inherits your terminal, so a vault CLI that
prompts works when you run `dam` yourself. `stale` makes reads pull first when the last pull is
older than that. `deadline` (default `"60s"`) bounds how long one helper answer may take; past it
the helper is killed and the command fails. Ctrl-C at any point ends `dam` the same way, killing
the helper and leaving the store as it was.

    dam pull

## Every day

    dam new "buy oat milk" --due tomorrow -p 1 --label errand
    dam ls due:today
    dam done 3f2a9c1
    dam add -A
    dam commit -m "morning triage"
    dam push

`dam edit <oid> --undone` reopens a task you completed by mistake, and
`dam restore <oid>...` throws a working change away, back to the last commit and out of the stage.

`dam status`, `dam diff`, `dam log` and `dam show` read the way their git namesakes do. `--json`
and `--toon` on any read give a program the same answer.

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

`|` or, `&` and, `!` not, parentheses group. `@label`, `pN`, `key:value`; an unknown key is looked
up as a category. Saved filters live under `[filter.<name>]` and run by name.

## Reading it from a program

`dam status --json` and `dam diff --json` answer in change documents, one per change:

    {"oid": "98d878...", "op": "update", "fields": ["subject", "due"], "kind": "task",
     "subject": "buy oat milk", "path": "work/", "labels": ["errand"], "done": false,
     "priority": 1, "due": "2026-09-25"}

`fields` names what the change touches. An update names what moved; a create names every field the
new object carries beyond its defaults; a delete names nothing and its row describes the object it
removed. The rest of the row is the state the change left behind, which is what a list renders.

Add `--full` to either command for `before` and `after`, the whole object on each side. The default
leaves them out because a client polling `status` per render pays for them on every poll.

## Errors

With `--json` or `--toon` a failure prints one document on standard error and nothing on standard
output:

    {"error": {"kind": "refused", "message": "98d8780 cannot be completed: child a9db854 is open",
     "oids": ["98d878013fb0e026d37170e7ceed6707192ae99a",
              "a9db854060d1943ef9eb9f6d7a8ac0b1ace45d77"]}}

`kind` is one of `refused`, `store`, `helper`, `credential`, `editor`, `parse`, `usage` and
`cancelled`, and `oids` names the objects the message names, in the order it names them. Standard
error carries that one document and nothing else, so parse the whole stream; a run that succeeds
writes nothing there. Without those flags you get the plain `dam: <message>` line instead.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | The command did what it was asked. |
| 1 | `dam` failed: a store, config, helper or io failure. |
| 2 | The command line was wrong: an unknown argument or subcommand, a flag value `dam` refuses to read, or an oid prefix that names more than one object. |
| 3 | Cancelled: you interrupted, or a prompt could not be answered. |
| 4 | `dam` refused by one of its own rules, and the message names the rule. |

Every rule `dam` keeps reports code 4, and nothing else does, so one mapping covers every verb.
