use clap::CommandFactory;
use serde_json::{Map, Value, json};

use crate::args::Cli;

const COMMANDS: [&str; 8] = [
    "new",
    "edit",
    "done",
    "mv",
    "rm",
    "list",
    "agenda",
    "filter create",
];

fn command_at(words: &str) -> clap::Command {
    words.split(' ').fold(Cli::command(), |command, word| {
        command.find_subcommand(word).unwrap().clone()
    })
}

fn flags_of(command: &clap::Command) -> Vec<Value> {
    command
        .get_arguments()
        .filter(|arg| !arg.is_hide_set() && !arg.is_global_set())
        .filter_map(|arg| {
            let help = arg.get_help().map(|h| h.to_string()).unwrap_or_default();
            Some(json!({
                "name": format!("--{}", arg.get_long()?),
                "value": arg.get_action().takes_values(),
                "help": help.lines().next().unwrap_or_default(),
            }))
        })
        .collect()
}

fn every_flag() -> String {
    let commands: Map<String, Value> = COMMANDS
        .iter()
        .map(|words| (words.to_string(), Value::from(flags_of(&command_at(words)))))
        .collect();
    serde_json::to_string_pretty(&commands).unwrap() + "\n"
}

#[test]
fn nvim_flags_json_holds_each_prompt_commands_flags() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../nvim/lua/damnit/kernel/flags.json");
    let text = every_flag();
    if std::env::var_os("DAM_WRITE_NVIM_FIXTURES").is_some() {
        std::fs::write(&path, &text).unwrap();
        return;
    }
    let kept = std::fs::read_to_string(&path).unwrap_or_default();
    assert_eq!(
        text,
        kept,
        "dam's flags drifted from {}; DAM_WRITE_NVIM_FIXTURES=1 rewrites it",
        path.display()
    );
}
