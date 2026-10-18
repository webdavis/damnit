use super::Cli;

const EVERY_HELP_TEXT: &str = "\
dam: about = Track tasks and events, staged and committed like git
dam json: help = Print JSON, and a failure as one error document on stderr
dam toon: help = Print TOON, a compact form for language models; errors as --json
dam config: help = Read the config from FILE. Default: ${XDG_CONFIG_HOME:-~/.config}/dam/config.toml
dam store: help = Open the store at FILE. Default: ${XDG_DATA_HOME:-~/.local/share}/dam/dam.db
dam no_pull: help = Never pull a stale remote first; answer from the local store
dam new: about = Create a task, or an event with --event
dam new subject: help = Title the new task or event
dam new event: help = Create an event instead of a task, which needs --start and --end
dam new path: help = Put it under PATH, such as work/clients/. Default: the root
dam new due: help = Set the due date: a date word, YYYY-MM-DD or YYYY-MM-DDTHH:MM
dam new priority: help = Set the priority, 1 (highest) to 4. Default: 4
dam new deadline: help = Set the deadline, a whole day read the way --due reads one
dam new labels: help = Add a label. Repeatable
dam new body: help = Set the body text
dam new start: help = Start the event at this time, read the way --due reads one
dam new end: help = End the event at this time, read the way --due reads one
dam done: about = Mark a task done
dam done oid: help = Pick the task by oid, or by a prefix of 4 or more hex characters
dam done force: help = Complete it even though something it waits on is still open
dam done interactive: help = Ask what happens to the open children and dependencies
dam done children: help = Decide what happens to open children instead of asking. Default: keep
dam done depends: help = Decide what happens to open dependency links instead of asking. Default: keep
dam edit: about = Change fields on an object
dam edit oid: help = Pick the object by oid, or by a prefix of 4 or more hex characters
dam edit editor: help = Open the object in your editor instead of passing flags
dam edit subject: help = Replace the subject
dam edit body: help = Replace the body text
dam edit priority: help = Set the priority, 1 (highest) to 4
dam edit due: help = Set the due date: a date word, YYYY-MM-DD or YYYY-MM-DDTHH:MM
dam edit no_due: help = Clear the due date
dam edit deadline: help = Set the deadline, a whole day read the way --due reads one
dam edit no_deadline: help = Clear the deadline
dam edit undone: help = Reopen a completed task
dam edit labels: help = Add a label. Repeatable
dam edit unlabels: help = Remove a label. Repeatable
dam edit depends: help = Make it wait on the task with this oid. Repeatable
dam edit undepends: help = Stop it waiting on the task with this oid. Repeatable
dam edit recurrence: help = Repeat the task by a rule such as \"every 2 weeks on mon\"
dam edit no_recurrence: help = Stop the task repeating
dam edit attach: help = Attach the task to the event with this oid
dam edit detach: help = Detach the task from its event
dam edit start: help = Move the event's start, read the way --due reads one
dam edit end: help = Move the event's end, read the way --due reads one
dam edit location: help = Set the event's location
dam edit no_location: help = Clear the event's location
dam mv: about = Move an object and its children to another path
dam mv oid: help = Pick the object by oid, or by a prefix of 4 or more hex characters
dam mv path: help = Move it under PATH, such as work/clients/
dam rm: about = Remove an object from the working layer
dam rm oid: help = Pick the object by oid, or by a prefix of 4 or more hex characters
dam add: about = Stage changes
dam add oids: help = Stage the changes to these objects, by oid or prefix
dam add all: help = Stage every change
dam reset: about = Unstage changes
dam reset oids: help = Unstage these objects, by oid or prefix. Default: everything staged
dam restore: about = Set objects back to their last committed state, staged or not
dam restore oids: help = Set back these objects, by oid or prefix
dam commit: about = Record the stage as a commit
dam commit message: help = Describe the commit
dam log: about = List commits, newest first
dam show: about = Show one object or one commit
dam show id: help = Pick an object by oid or a commit by id, full or prefixed
dam status: about = Show what changed, what is staged and what a remote reported
dam status full: help = Embed the whole object before and after each change, in --json and --toon
dam diff: about = Show unstaged changes, or staged ones with --staged
dam diff staged: help = Show the staged changes instead of the unstaged ones
dam diff full: help = Embed the whole object before and after each change, in --json and --toon
dam ls: about = List objects, optionally by query or saved filter
dam ls query: help = Narrow the list by a query, such as due:today, or a saved filter's name
dam agenda: about = Show events over a window, each with whether it holds time
dam agenda query: help = Narrow the events by a query or saved filter, read the way `ls` reads one
dam agenda from: help = Open the window at a date word, YYYY-MM-DD or YYYY-MM-DDTHH:MM. Default: now
dam agenda to: help = Close the window here, exclusive. Default: 24 hours after it opens
dam agenda max_age: help = Refuse when REMOTE last pulled over DURATION ago, such as 15m. Repeatable
dam category: about = Read the label categories the config declares
dam category list: about = List declared categories with their values and whether they are exclusive
dam filter: about = Read the saved filters the config declares
dam filter list: about = List saved filters with their queries
dam remote: about = Manage remotes
dam remote add: about = Add a remote to the config file
dam remote add name: help = Name the remote, such as todoist
dam remote add url: help = Point it at a helper URL, such as todoist:: or gcal::primary
dam remote list: about = List configured remotes
dam push: about = Send unpushed commits to a remote, or to every remote
dam push remote: help = Push to this remote alone. Default: every remote
dam pull: about = Fetch and merge from a remote, or from every remote
dam pull remote: help = Pull from this remote alone. Default: every remote
dam resolve: about = Settle a conflict for one object
dam resolve oid: help = Pick the conflicted object by oid, or by a prefix of 4 or more hex characters
dam resolve ours: help = Keep the local side
dam resolve theirs: help = Keep the remote's side
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

fn every_help_page(command: &clap::Command, out: &mut Vec<(String, String)>) {
    let name = command.get_bin_name().unwrap_or_default().to_string();
    out.push((name, command.clone().render_help().to_string()));
    for sub in command.get_subcommands() {
        every_help_page(sub, out);
    }
}

#[test]
fn no_help_line_passes_100_columns() {
    use clap::CommandFactory;
    let mut root = Cli::command();
    root.build();
    let mut pages = Vec::new();
    every_help_page(&root, &mut pages);
    assert!(pages.iter().any(|(name, _)| name == "dam remote add"));
    for (name, page) in pages {
        for line in page.lines() {
            assert!(line.chars().count() <= 100, "{name}: {line}");
        }
    }
}
