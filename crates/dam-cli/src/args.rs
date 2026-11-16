mod config;
mod dispositions;
mod doctor;
mod editing;
mod global_flags;
pub(crate) mod hook;
mod init;
mod new;
mod planning;
mod reading;
mod remotes;
mod scope;
pub(crate) mod selection;
mod set;
mod staging;
mod test;
mod tokens;
pub(crate) mod values;
mod writing;

use clap::Subcommand;

pub(crate) use config::{ConfigArgs, ConfigCommand};
pub(crate) use doctor::{CheckName, DoctorArgs};
pub(crate) use editing::EditArgs;
pub(crate) use global_flags::Cli;
pub(crate) use init::InitArgs;
pub(crate) use new::NewArgs;
pub(crate) use planning::{DayArgs, DayCommand, RemindersArgs};
pub(crate) use reading::{
    AgendaArgs, CategoryArgs, CategoryCommand, FilterArgs, FilterCommand, ListArgs,
};
pub(crate) use remotes::{
    PullArgs, PushArgs, RemoteArgs, RemoteCommand, ResolveArgs, SyncArgs, SyncCommand,
};
pub(crate) use scope::ScopeArgs;
pub(crate) use set::SetArgs;
pub(crate) use staging::{
    AddArgs, CommitArgs, DiffArgs, ResetArgs, RestoreArgs, RevertArgs, ShowArgs, StatusArgs,
};
pub(crate) use test::{PartName, TestArgs};
pub(crate) use tokens::{CreateArgs, TokenArgs, TokenCommand};
pub(crate) use writing::{DoneArgs, MvArgs, RmArgs};

#[derive(Subcommand, Debug)]
pub(crate) enum Command {
    #[command(
        about = "Make this folder a project: write .dam.lua and create the project's task",
        after_help = init::INIT_HELP
    )]
    Init(InitArgs),
    #[command(about = "Create a task, or an event with --kind event")]
    New(NewArgs),
    #[command(about = "Mark a task done")]
    Done(DoneArgs),
    #[command(about = "Change fields on an object")]
    Edit(EditArgs),
    #[command(about = "Move an object and its children to another path")]
    Mv(MvArgs),
    #[command(about = "Remove an object from the working layer")]
    Rm(RmArgs),
    #[command(about = "Stage changes")]
    Add(AddArgs),
    #[command(about = "Unstage changes")]
    Reset(ResetArgs),
    #[command(about = "Set objects back to their last committed state, staged or not")]
    Restore(RestoreArgs),
    #[command(
        about = "Set back what a commit changed, as working changes to review",
        after_help = staging::REVERT_HELP
    )]
    Revert(RevertArgs),
    #[command(about = "Record the stage as a commit")]
    Commit(CommitArgs),
    #[command(about = "List commits, newest first")]
    Log,
    #[command(about = "Show one object or one commit")]
    Show(ShowArgs),
    #[command(about = "Show what changed, what is staged and what a remote reported")]
    Status(StatusArgs),
    #[command(about = "Show unstaged changes, or staged ones with --staged")]
    Diff(DiffArgs),
    #[command(
        about = "List tasks that are not done, or what the flags select",
        after_help = selection::LIST_HELP
    )]
    List(ListArgs),
    #[command(about = "Show events over a window, each with whether it holds time")]
    Agenda(AgendaArgs),
    #[command(about = "List the reminders that fall due in a window, by default today until now")]
    Reminders(RemindersArgs),
    #[command(about = "Read and record the day's plan, proposals, decisions and task states")]
    Day(DayArgs),
    #[command(about = "List, create, remove and migrate the label categories config.lua declares")]
    Category(CategoryArgs),
    #[command(about = "List, create and remove the saved filters config.lua declares")]
    Filter(FilterArgs),
    #[command(about = "List the hooks in the hooks folder, or run one over listed tasks")]
    Hook(hook::HookArgs),
    #[command(about = "Manage remotes")]
    Remote(RemoteArgs),
    #[command(about = "Send unpushed commits to a remote, or to every remote")]
    Push(PushArgs),
    #[command(about = "Fetch and merge from a remote, or from every remote")]
    Pull(PullArgs),
    #[command(about = "Settle a conflict for one object")]
    Resolve(ResolveArgs),
    #[command(
        about = "Sync a remote",
        after_help = "Examples:\n  dam sync run todoist"
    )]
    Sync(SyncArgs),
    #[command(about = "Manage the tokens tools and agents run dam with")]
    Token(TokenArgs),
    #[command(
        about = "Show and check the config",
        after_help = "Examples:\n  dam config list\n  dam config get remote.todoist.deadline -o json\n  dam config validate"
    )]
    Config(ConfigArgs),
    #[command(
        about = "Read this machine, say what is wrong and how to fix it, and contact nothing",
        after_help = doctor::DOCTOR_HELP
    )]
    Doctor(DoctorArgs),
    #[command(
        about = "Make each part do its real job once and say whether it worked",
        after_help = "It contacts each remote, and runs credential functions, which may prompt.
The filters part runs each saved filter once over the store.
It writes nothing: not the store, not the config, not a remote.
It exits 1 when a part finds an issue, and prints the report to stdout either way.

Examples:
  dam test
  dam test credentials
  dam test filters
  dam test -o json"
    )]
    Test(TestArgs),
}

#[cfg(test)]
mod help_text_tests;
#[cfg(test)]
mod labels_tests;
#[cfg(test)]
mod output_flag_tests;
#[cfg(test)]
mod parent_tests;
#[cfg(test)]
mod tests;
