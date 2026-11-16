use clap::CommandFactory;
use dam_domain::select_field;
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

fn value_of(arg: &clap::Arg) -> &'static str {
    match arg.get_num_args() {
        Some(range) if !range.takes_values() => "absent",
        Some(range) if range.min_values() == 0 => "optional",
        Some(_) => "required",
        None if arg.get_action().takes_values() => "required",
        None => "absent",
    }
}

fn values_of(arg: &clap::Arg) -> Option<Vec<String>> {
    let known: Vec<String> = arg
        .get_possible_values()
        .iter()
        .map(|value| value.get_name().to_string())
        .collect();
    if !known.is_empty() {
        return Some(known);
    }
    (value_of(arg) == "optional")
        .then(|| select_field(arg.get_id().as_str()).map(|field| field.shapes()))
        .flatten()
}

fn flags_of(command: &clap::Command) -> Vec<Value> {
    command
        .get_arguments()
        .filter(|arg| !arg.is_hide_set() && !arg.is_global_set())
        .filter_map(|arg| {
            let help = arg.get_help().map(|h| h.to_string()).unwrap_or_default();
            let mut flag = json!({
                "name": format!("--{}", arg.get_long()?),
                "value": value_of(arg),
                "help": help.lines().next().unwrap_or_default(),
            });
            if let Some(values) = values_of(arg) {
                flag["values"] = json!(values);
            }
            Some(flag)
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

fn flag_named(command: &str, name: &str) -> Value {
    let flags = flags_of(&command_at(command));
    flags
        .into_iter()
        .find(|flag| flag["name"] == name)
        .unwrap_or_else(|| panic!("dam {command} has no {name}"))
}

#[test]
fn nvim_flags_json_says_whether_a_flags_value_is_required_optional_or_absent() {
    let labels = flag_named("list", "--labels");
    let global = flag_named("list", "--global");
    assert_eq!(labels["value"], "required");
    assert_eq!(global["value"], "absent");
    assert_eq!(labels.get("values"), None);
}

#[test]
fn nvim_flags_json_names_the_values_a_yes_no_flag_may_take() {
    for name in ["--done", "--overdue", "--blocked"] {
        let flag = flag_named("list", name);
        assert_eq!(flag["value"], "optional", "{name}");
        assert_eq!(flag["values"], json!(["true", "false", "any"]), "{name}");
    }
}
