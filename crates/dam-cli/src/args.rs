//! The command surface. Each group of verbs declares its own arguments in
//! a module of its own; this file is the table clap parses them through.

mod dispositions;
mod reading;
mod remotes;
mod staging;
mod writing;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

pub(crate) use reading::{CategoryArgs, CategoryCommand, FilterArgs, FilterCommand, LsArgs};
pub(crate) use remotes::{PullArgs, PushArgs, RemoteArgs, RemoteCommand, ResolveArgs};
pub(crate) use staging::{
    AddArgs, CommitArgs, DiffArgs, ResetArgs, RestoreArgs, ShowArgs, StatusArgs,
};
pub(crate) use writing::{DoneArgs, EditArgs, MvArgs, NewArgs, RmArgs};

#[derive(Parser, Debug)]
#[command(
    name = "dam",
    version,
    about = "tasks and events, staged and committed like git"
)]
pub(crate) struct Cli {
    /// Print the result as JSON, and a failure as one error document on stderr.
    #[arg(long, global = true, conflicts_with = "toon")]
    pub(crate) json: bool,
    /// Print the result as TOON, a compact form for language models; errors as --json.
    #[arg(long, global = true)]
    pub(crate) toon: bool,
    #[arg(long, global = true, env = "DAM_CONFIG", value_name = "FILE")]
    pub(crate) config: Option<PathBuf>,
    #[arg(long, global = true, env = "DAM_STORE", value_name = "FILE")]
    pub(crate) store: Option<PathBuf>,
    /// Answer from the local store alone: no read pulls a stale remote.
    #[arg(long, global = true)]
    pub(crate) no_pull: bool,
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Command {
    /// Create a task, or an event with --event.
    New(NewArgs),
    /// Mark a task done.
    Done(DoneArgs),
    /// Change fields on an object.
    Edit(EditArgs),
    /// Move an object and its children to another path.
    Mv(MvArgs),
    /// Remove an object from the working layer.
    Rm(RmArgs),
    /// Stage changes.
    Add(AddArgs),
    /// Unstage changes.
    Reset(ResetArgs),
    /// Set objects back to their last committed state, staged or not.
    Restore(RestoreArgs),
    /// Record the stage as a commit.
    Commit(CommitArgs),
    /// List commits, newest first.
    Log,
    /// Show one object or one commit.
    Show(ShowArgs),
    /// Working versus stage versus last commit, plus remote notices.
    Status(StatusArgs),
    /// Unstaged changes, or staged with --staged.
    Diff(DiffArgs),
    /// List objects, optionally by query or saved filter.
    Ls(LsArgs),
    /// The label categories config declares.
    Category(CategoryArgs),
    /// The saved filters config declares.
    Filter(FilterArgs),
    /// Manage remotes.
    Remote(RemoteArgs),
    /// Send unpushed commits to a remote, or to every remote.
    Push(PushArgs),
    /// Fetch and merge from a remote, or from every remote.
    Pull(PullArgs),
    /// Settle a conflict for one object.
    Resolve(ResolveArgs),
}

#[cfg(test)]
mod tests;
