use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub(crate) struct RemoteArgs {
    #[command(subcommand)]
    pub(crate) command: RemoteCommand,
}

#[derive(Subcommand, Debug)]
pub(crate) enum RemoteCommand {
    #[command(about = "Add a remote to config.lua")]
    Add {
        #[arg(help = "Name the remote, such as todoist")]
        name: String,
        #[arg(help = "Point it at a helper URL, such as todoist:: or gcal::primary")]
        url: String,
    },
    #[command(about = "List configured remotes")]
    List,
}

#[derive(Args, Debug)]
pub(crate) struct SyncArgs {
    #[command(subcommand)]
    pub(crate) command: Option<SyncCommand>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum SyncCommand {
    #[command(
        about = "Pull from and push to each remote whose sync_every has passed since its last pull",
        after_help = "A remote that takes no kinds is pulled from alone.\nA failed sync leaves one notice in dam status, until a sync of that remote works.\nIt exits 1 when a remote's sync failed, and prints the report to stdout either way.\n\nExamples:\n  dam sync run\n  dam sync run --json"
    )]
    Run,
}

#[derive(Args, Debug)]
pub(crate) struct PushArgs {
    #[arg(help = "Push to this remote alone. Default: every remote")]
    pub(crate) remote: Option<String>,
}

#[derive(Args, Debug)]
pub(crate) struct PullArgs {
    #[arg(help = "Pull from this remote alone. Default: every remote")]
    pub(crate) remote: Option<String>,
}

#[derive(Args, Debug)]
pub(crate) struct ResolveArgs {
    #[arg(help = "Pick the conflicted object by oid, or by a prefix of 4 or more hex characters")]
    pub(crate) oid: String,
    #[command(flatten)]
    pub(crate) side: Side,
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
pub(crate) struct Side {
    #[arg(long, help = "Keep the local side")]
    pub(crate) ours: bool,
    #[arg(long, help = "Keep the remote's side")]
    pub(crate) theirs: bool,
}
