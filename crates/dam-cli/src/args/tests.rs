use super::*;
use clap::Parser;
use dam_domain::{ChildDisposition, DependencyDisposition};

#[test]
fn new_with_flags_parses() {
    let cli = Cli::try_parse_from([
        "dam", "new", "buy milk", "--path", "inbox/", "--due", "tomorrow", "-p", "1", "--label",
        "errand",
    ])
    .unwrap();
    match cli.command {
        Command::New(a) => {
            assert_eq!(a.subject, "buy milk");
            assert_eq!(a.path, "inbox/");
            assert_eq!(a.due.as_deref(), Some("tomorrow"));
            assert_eq!(a.priority, Some(1));
            assert_eq!(a.labels, vec!["errand".to_string()]);
        }
        _ => panic!("wrong command"),
    }
}

#[test]
fn json_and_toon_are_global_and_exclusive() {
    let cli = Cli::try_parse_from(["dam", "ls", "--json"]).unwrap();
    assert!(cli.json);
    assert!(Cli::try_parse_from(["dam", "ls", "--json", "--toon"]).is_err());
}

#[test]
fn no_pull_is_global_and_defaults_to_pulling() {
    assert!(!Cli::try_parse_from(["dam", "ls"]).unwrap().no_pull);
    assert!(
        Cli::try_parse_from(["dam", "ls", "--no-pull"])
            .unwrap()
            .no_pull
    );
    assert!(
        Cli::try_parse_from(["dam", "show", "abcd", "--no-pull"])
            .unwrap()
            .no_pull
    );
}

#[test]
fn resolve_needs_exactly_one_side() {
    assert!(Cli::try_parse_from(["dam", "resolve", "abc", "--ours"]).is_ok());
    assert!(Cli::try_parse_from(["dam", "resolve", "abc"]).is_err());
    assert!(Cli::try_parse_from(["dam", "resolve", "abc", "--ours", "--theirs"]).is_err());
}

#[test]
fn edit_takes_undone_beside_the_other_clearing_flags() {
    let cli = Cli::try_parse_from(["dam", "edit", "abcd", "--undone"]).unwrap();
    match cli.command {
        Command::Edit(a) => assert!(a.undone),
        _ => panic!("wrong command"),
    }
}

#[test]
fn each_disposition_word_parses_to_its_variant() {
    let done = |args: &[&str]| {
        let mut argv = vec!["dam", "done", "abcd", "--force"];
        argv.extend_from_slice(args);
        match Cli::try_parse_from(argv).unwrap().command {
            Command::Done(a) => a,
            _ => panic!("wrong command"),
        }
    };
    assert_eq!(
        done(&["--children", "up"]).children,
        Some(ChildDisposition::Up)
    );
    assert_eq!(
        done(&["--children", "keep"]).children,
        Some(ChildDisposition::Keep)
    );
    assert_eq!(
        done(&["--children", "into:leftovers"]).children,
        Some(ChildDisposition::Into("leftovers".into()))
    );
    assert_eq!(
        done(&["--depends", "drop"]).depends,
        Some(DependencyDisposition::Drop)
    );
    assert_eq!(
        done(&["--depends", "keep"]).depends,
        Some(DependencyDisposition::Keep)
    );
    assert_eq!(done(&[]).children, None);
    assert_eq!(done(&[]).depends, None);
}

#[test]
fn a_disposition_outside_the_set_is_a_usage_error_naming_the_accepted_words_and_help() {
    for (bad, accepted) in [
        (vec!["--children", "sideways"], "up, keep, into:<name>"),
        (vec!["--children", "into:"], "up, keep, into:<name>"),
        (vec!["--depends", "maybe"], "drop, keep"),
    ] {
        let mut argv = vec!["dam", "done", "abcd", "--force"];
        argv.extend_from_slice(&bad);
        let err = Cli::try_parse_from(argv).unwrap_err();
        assert_eq!(err.exit_code(), 2, "{bad:?}");
        let rendered = err.to_string();
        assert!(rendered.contains(bad[0]), "{bad:?}: {rendered}");
        assert!(rendered.contains(accepted), "{bad:?}: {rendered}");
        assert!(rendered.contains("--help"), "{bad:?}: {rendered}");
    }
}

#[test]
fn a_disposition_beside_an_explicit_interactive_is_refused() {
    for flag in [vec!["--children", "keep"], vec!["--depends", "drop"]] {
        let mut argv = vec!["dam", "done", "abcd", "--force", "--interactive"];
        argv.extend_from_slice(&flag);
        let err = Cli::try_parse_from(argv).unwrap_err();
        assert_eq!(err.exit_code(), 2, "{flag:?}");
        assert!(err.to_string().contains("--interactive"), "{flag:?}: {err}");
    }
}

#[test]
fn a_disposition_without_force_is_refused() {
    assert!(Cli::try_parse_from(["dam", "done", "abcd", "--children", "keep"]).is_err());
    assert!(Cli::try_parse_from(["dam", "done", "abcd", "--depends", "drop"]).is_err());
}

#[test]
fn restore_needs_at_least_one_oid() {
    assert!(Cli::try_parse_from(["dam", "restore", "abcd"]).is_ok());
    assert!(Cli::try_parse_from(["dam", "restore"]).is_err());
}

#[test]
fn add_all_and_oids_are_alternatives() {
    assert!(Cli::try_parse_from(["dam", "add", "-A"]).is_ok());
    assert!(Cli::try_parse_from(["dam", "add", "abc", "def"]).is_ok());
    assert!(Cli::try_parse_from(["dam", "add"]).is_err());
}

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
