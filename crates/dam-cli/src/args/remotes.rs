//! Remotes and the commands that move objects across them.

use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub(crate) struct RemoteArgs {
    #[command(subcommand)]
    pub(crate) command: RemoteCommand,
}

#[derive(Subcommand, Debug)]
pub(crate) enum RemoteCommand {
    /// Add a remote to the config file.
    Add { name: String, url: String },
    /// List configured remotes.
    List,
}

#[derive(Args, Debug)]
pub(crate) struct PushArgs {
    pub(crate) remote: Option<String>,
}

#[derive(Args, Debug)]
pub(crate) struct PullArgs {
    pub(crate) remote: Option<String>,
}

#[derive(Args, Debug)]
pub(crate) struct ResolveArgs {
    pub(crate) oid: String,
    #[arg(long, conflicts_with = "theirs", required_unless_present = "theirs")]
    pub(crate) ours: bool,
    #[arg(long)]
    pub(crate) theirs: bool,
}
