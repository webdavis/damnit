mod config;
mod dispositions;
mod editing;
mod global_flags;
mod new;
mod planning;
mod reading;
mod remotes;
mod staging;
mod tokens;
mod values;
mod writing;

use clap::Subcommand;

pub(crate) use config::{ConfigArgs, ConfigCommand};
pub(crate) use editing::EditArgs;
pub(crate) use global_flags::Cli;
pub(crate) use new::NewArgs;
pub(crate) use planning::{DayArgs, DayCommand, RemindersArgs};
pub(crate) use reading::{
    AgendaArgs, AuthorKind, CategoryArgs, CategoryCommand, FilterArgs, FilterCommand, LsArgs,
};
pub(crate) use remotes::{PullArgs, PushArgs, RemoteArgs, RemoteCommand, ResolveArgs};
pub(crate) use staging::{
    AddArgs, CommitArgs, DiffArgs, ResetArgs, RestoreArgs, ShowArgs, StatusArgs,
};
pub(crate) use tokens::{CreateArgs, TokenArgs, TokenCommand};
pub(crate) use writing::{DoneArgs, MvArgs, RmArgs};

#[derive(Subcommand, Debug)]
pub(crate) enum Command {
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
    #[command(about = "List objects, optionally by query or saved filter")]
    Ls(LsArgs),
    #[command(about = "Show events over a window, each with whether it holds time")]
    Agenda(AgendaArgs),
    #[command(about = "List the reminders that fall due in a window, by default today until now")]
    Reminders(RemindersArgs),
    #[command(about = "Read and record the day's plan, proposals, decisions and task states")]
    Day(DayArgs),
    #[command(about = "Read the label categories config.lua declares")]
    Category(CategoryArgs),
    #[command(about = "Read the saved filters config.lua declares")]
    Filter(FilterArgs),
    #[command(about = "Manage remotes")]
    Remote(RemoteArgs),
    #[command(about = "Send unpushed commits to a remote, or to every remote")]
    Push(PushArgs),
    #[command(about = "Fetch and merge from a remote, or from every remote")]
    Pull(PullArgs),
    #[command(about = "Settle a conflict for one object")]
    Resolve(ResolveArgs),
    #[command(about = "Manage the tokens tools and agents run dam with")]
    Token(TokenArgs),
    #[command(about = "Check the config files")]
    Config(ConfigArgs),
}

#[cfg(test)]
mod help_text_tests;
#[cfg(test)]
mod tests;
