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
        after_help = "A remote that takes no kinds is pulled from alone.\nA failed sync leaves one notice in dam status, until a sync of that remote works.\nIt exits 1 when a remote's sync failed, and prints the report to stdout either way.\nWith --record-last-run the report, or the error, replaces the last-run file instead.\n\nExamples:\n  dam sync run\n  dam sync run --json"
    )]
    Run {
        #[arg(
            long,
            help = "Write the report or the error to the last-run file instead of printing it"
        )]
        record_last_run: bool,
    },
    #[command(
        about = "Have the system run dam sync run every minute for this user",
        after_help = "On macOS it installs a launchd agent; on Linux, a systemd user timer.\nIt starts a first sync at once, then one every minute.\nThe entry runs this dam with the config and store in use now.\nEach run replaces the last-run file it prints.\nWhere neither runs, it refuses and prints the crontab line to add.\n\nExamples:\n  dam sync enable\n  dam sync enable --json"
    )]
    Enable,
    #[command(
        about = "Stop the system running dam sync run and remove its entry",
        after_help = "Examples:\n  dam sync disable\n  dam sync disable --json"
    )]
    Disable,
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
