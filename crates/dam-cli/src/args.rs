mod dispositions;
mod reading;
mod remotes;
mod staging;
mod values;
mod writing;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

pub(crate) use reading::{
    AgendaArgs, CategoryArgs, CategoryCommand, FilterArgs, FilterCommand, LsArgs,
};
pub(crate) use remotes::{PullArgs, PushArgs, RemoteArgs, RemoteCommand, ResolveArgs};
pub(crate) use staging::{
    AddArgs, CommitArgs, DiffArgs, ResetArgs, RestoreArgs, ShowArgs, StatusArgs,
};
pub(crate) use writing::{DoneArgs, EditArgs, MvArgs, NewArgs, RmArgs};

#[derive(Parser, Debug)]
#[command(
    name = "dam",
    version,
    about = "Track tasks and events, staged and committed like git",
    max_term_width = 100
)]
pub(crate) struct Cli {
    #[arg(
        long,
        global = true,
        conflicts_with = "toon",
        help = "Print JSON, and a failure as one error document on stderr"
    )]
    pub(crate) json: bool,
    #[arg(
        long,
        global = true,
        help = "Print TOON, a compact form for language models; errors as --json"
    )]
    pub(crate) toon: bool,
    #[arg(
        long,
        global = true,
        env = "DAM_CONFIG",
        hide_env_values = true,
        value_name = "FILE",
        help = "Read the config from FILE. Default: ${XDG_CONFIG_HOME:-~/.config}/dam/config.toml"
    )]
    pub(crate) config: Option<PathBuf>,
    #[arg(
        long,
        global = true,
        env = "DAM_STORE",
        hide_env_values = true,
        value_name = "FILE",
        help = "Open the store at FILE. Default: ${XDG_DATA_HOME:-~/.local/share}/dam/dam.db"
    )]
    pub(crate) store: Option<PathBuf>,
    #[arg(
        long,
        global = true,
        help = "Never pull a stale remote first; answer from the local store"
    )]
    pub(crate) no_pull: bool,
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Command {
    #[command(about = "Create a task, or an event with --event")]
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
    #[command(about = "Read the label categories the config declares")]
    Category(CategoryArgs),
    #[command(about = "Read the saved filters the config declares")]
    Filter(FilterArgs),
    #[command(about = "Manage remotes")]
    Remote(RemoteArgs),
    #[command(about = "Send unpushed commits to a remote, or to every remote")]
    Push(PushArgs),
    #[command(about = "Fetch and merge from a remote, or from every remote")]
    Pull(PullArgs),
    #[command(about = "Settle a conflict for one object")]
    Resolve(ResolveArgs),
}

#[cfg(test)]
mod help_text_tests;
#[cfg(test)]
mod tests;
