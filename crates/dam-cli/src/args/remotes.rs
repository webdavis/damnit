use std::time::Duration;

use clap::{Args, Subcommand};
use dam_application::{ConfiguredDuration, RemoteSync};

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
        about = "Pull from one remote, then push its committed changes, now",
        after_help = "It runs whatever sync.allowed says: a sync you type is not background sync.
Newer or uncommitted work is never replaced. A conflict is left for dam status and dam resolve,
and the push goes on past it.

Examples:
  dam sync run todoist
  dam sync run todoist --json
  dam sync run todoist --when-due"
    )]
    Run {
        #[arg(help = "Sync this remote")]
        remote: String,
        #[arg(
            long,
            help = "Sync only when the interval has passed since the last sync; otherwise do nothing"
        )]
        when_due: bool,
        #[arg(
            long,
            requires = "when_due",
            value_parser = interval_of_at_least_a_minute,
            help = "Use this interval in place of remote.<name>.sync.interval, such as 5m"
        )]
        interval: Option<Duration>,
    },
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

pub(crate) fn interval_of_at_least_a_minute(text: &str) -> Result<Duration, String> {
    ConfiguredDuration::parse(text)
        .map(|d| d.value)
        .filter(|d| *d >= RemoteSync::FLOOR)
        .ok_or_else(|| {
            format!(
                "give a whole number and s, m or h of at least {}, such as 5m",
                RemoteSync::FLOOR_TEXT
            )
        })
}

#[cfg(test)]
mod tests;
