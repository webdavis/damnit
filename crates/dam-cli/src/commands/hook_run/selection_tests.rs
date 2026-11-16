use clap::Parser;

use super::run;
use crate::args::hook::{HookCommand, HookRunArgs};
use crate::args::{Cli, Command};
use crate::testing::context;

fn hook_run_args(words: &[&str]) -> HookRunArgs {
    match Cli::try_parse_from(["dam", "hook", "run"].iter().chain(words)) {
        Ok(Cli {
            command: Command::Hook(hook),
            ..
        }) => match hook.command {
            Some(HookCommand::Run(args)) => args,
            other => panic!("not hook run: {other:?}"),
        },
        other => panic!("not hook run: {other:?}"),
    }
}

#[test]
fn hook_run_reads_the_selection_flags() {
    let args = hook_run_args(&["task_new", "--labels", "bug", "--due", "..today"]);
    assert_eq!(args.selection.typed.fields.len(), 2);
}

#[test]
fn a_bad_selection_value_names_hook_run_and_its_hook_in_the_command_to_run() {
    let err = run(&context(), hook_run_args(&["task_new", "--due", "tomorow"])).unwrap_err();
    assert_eq!(err.exit_code(), 2);
    assert!(
        err.to_string()
            .ends_with("; run dam hook run task_new --due today"),
        "{err}"
    );
}
