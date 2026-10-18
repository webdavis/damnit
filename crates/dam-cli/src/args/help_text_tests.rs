use super::Cli;

const EVERY_HELP_TEXT: &str = "\
dam: about = tasks and events, staged and committed like git
dam json: help = Print the result as JSON, and a failure as one error document on stderr
dam toon: help = Print the result as TOON, a compact form for language models; errors as --json
dam no_pull: help = Answer from the local store alone: no read pulls a stale remote
dam new: about = Create a task, or an event with --event
dam done: about = Mark a task done
dam done force: help = Complete it even though something it waits on is still open
dam done interactive: help = Ask what happens to the open children and dependencies
dam done children: help = What happens to open children, instead of asking. Default: keep
dam done depends: help = What happens to open dependency links, instead of asking. Default: keep
dam edit: about = Change fields on an object
dam edit editor: help = Open the object in your editor instead of passing flags
dam edit undone: help = Reopen a completed task
dam mv: about = Move an object and its children to another path
dam rm: about = Remove an object from the working layer
dam add: about = Stage changes
dam reset: about = Unstage changes
dam restore: about = Set objects back to their last committed state, staged or not
dam restore oids: help = The objects to set back, named in full or by oid prefix
dam commit: about = Record the stage as a commit
dam log: about = List commits, newest first
dam show: about = Show one object or one commit
dam show id: help = An object oid or a commit id, full or prefixed
dam status: about = Working versus stage versus last commit, plus remote notices
dam status full: help = Embed the whole before and after object in every change; shapes --json and --toon only
dam diff: about = Unstaged changes, or staged with --staged
dam diff full: help = Embed the whole before and after object in every change; shapes --json and --toon only
dam ls: about = List objects, optionally by query or saved filter
dam agenda: about = Events over a window, each with whether it holds time
dam agenda query: help = A query or saved filter that narrows the events, read the way `ls` reads one
dam agenda from: help = Where the window opens: a date word, YYYY-MM-DD or YYYY-MM-DDTHH:MM. Default: now
dam agenda to: help = Where the window closes, exclusive. Default: 24 hours after it opens
dam agenda max_age: help = Refuse to answer when REMOTE last pulled longer ago than DURATION (such as 90s, 15m or 2h), or never. Repeatable
dam category: about = The label categories config declares
dam category list: about = List declared categories with their values and whether they are exclusive
dam filter: about = The saved filters config declares
dam filter list: about = List saved filters with their queries
dam remote: about = Manage remotes
dam remote add: about = Add a remote to the config file
dam remote list: about = List configured remotes
dam push: about = Send unpushed commits to a remote, or to every remote
dam pull: about = Fetch and merge from a remote, or from every remote
dam resolve: about = Settle a conflict for one object
";

fn every_help_text(command: &clap::Command, path: &str, out: &mut String) {
    let path = format!("{path} {}", command.get_name()).trim().to_string();
    let texts = [
        ("about", command.get_about()),
        ("long_about", command.get_long_about()),
    ];
    for (key, text) in texts {
        if let Some(text) = text {
            out.push_str(&format!("{path}: {key} = {text}\n"));
        }
    }
    for arg in command.get_arguments() {
        let texts = [("help", arg.get_help()), ("long_help", arg.get_long_help())];
        for (key, text) in texts {
            if let Some(text) = text {
                out.push_str(&format!("{path} {}: {key} = {text}\n", arg.get_id()));
            }
        }
    }
    for sub in command.get_subcommands() {
        every_help_text(sub, &path, out);
    }
}

#[test]
fn every_command_and_flag_keeps_its_help_text() {
    use clap::CommandFactory;
    let mut texts = String::new();
    every_help_text(&Cli::command(), "", &mut texts);
    assert_eq!(texts, EVERY_HELP_TEXT);
}
