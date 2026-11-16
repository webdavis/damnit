use clap::{Args, Subcommand, ValueEnum};
use dam_application::HookEvent;
use dam_domain::Has;

use super::selection::SelectArgs;

#[derive(Args, Debug)]
pub(crate) struct HookArgs {
    #[command(subcommand)]
    pub(crate) command: Option<HookCommand>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum HookCommand {
    #[command(
        about = "List the hooks in the hooks folder, each with its file, setting and whether it loads"
    )]
    List,
    #[command(
        about = "Run a hook once for each task a listing holds, leaving its changes as working changes",
        after_help = "Each task is handed to the hook as new: task_pulled gets no before and change = new.
Nothing happened to these tasks, so a rule that compares before and after does nothing here;
only a rule that looks at the task as it is works.
A hook that prints, sends a notice or runs a program does so once for each listed task.
Nothing retries by itself: a task the hook failed on or did not reach stays as it is until you
run this again, on a narrower listing if need be.
task_pulled runs only for tasks tracked on a remote; the others count as not reached.
It contacts no remote. Read the changes with dam diff; dam restore undoes them.

Examples:
  dam hook run task_changed
  dam hook run task_pulled --labels waiting
  dam hook run task_new --due today --done false -o json"
    )]
    Run(HookRunArgs),
}

#[derive(Args, Debug)]
#[command(mut_args = super::selection::shown_for(Has::Task))]
pub(crate) struct HookRunArgs {
    #[arg(value_enum, help = "Name the hook to run by its event")]
    pub(crate) hook: HookName,
    #[command(flatten)]
    pub(crate) selection: SelectArgs,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(crate) enum HookName {
    #[value(name = "task_new")]
    New,
    #[value(name = "task_changed")]
    Changed,
    #[value(name = "task_done")]
    Done,
    #[value(name = "task_pulled")]
    Pulled,
}

impl From<HookName> for HookEvent {
    fn from(name: HookName) -> HookEvent {
        match name {
            HookName::New => HookEvent::TaskNew,
            HookName::Changed => HookEvent::TaskChanged,
            HookName::Done => HookEvent::TaskDone,
            HookName::Pulled => HookEvent::TaskPulled,
        }
    }
}
