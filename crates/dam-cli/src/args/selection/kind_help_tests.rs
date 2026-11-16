use clap::CommandFactory;

use crate::args::Cli;

fn help(words: &[&str]) -> String {
    let mut command = Cli::command();
    command.build();
    let leaf = words.iter().fold(command, |c, word| {
        c.find_subcommand(word).expect("a command").clone()
    });
    leaf.clone().render_help().to_string()
}

#[test]
fn agenda_help_shows_shared_and_event_fields_and_hides_task_fields() {
    let text = help(&["agenda"]);
    for shown in ["--labels", "--start", "--location"] {
        assert!(text.contains(shown), "{shown}: {text}");
    }
    for hidden in ["--due", "--done", "--priority"] {
        assert!(!text.contains(hidden), "{hidden}: {text}");
    }
}

#[test]
fn hook_run_help_shows_shared_and_task_fields_and_hides_event_fields() {
    let text = help(&["hook", "run"]);
    for shown in ["--labels", "--due", "--priority"] {
        assert!(text.contains(shown), "{shown}: {text}");
    }
    for hidden in ["--start", "--location", "--attendees-email"] {
        assert!(!text.contains(hidden), "{hidden}: {text}");
    }
}

#[test]
fn list_help_shows_every_field() {
    let text = help(&["list"]);
    for shown in ["--labels", "--due", "--start"] {
        assert!(text.contains(shown), "{shown}: {text}");
    }
}
