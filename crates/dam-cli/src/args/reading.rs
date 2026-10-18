use std::time::Duration;

use clap::{Args, Subcommand};
use dam_application::RemoteName;

use super::values::max_age_arg;

#[derive(Args, Debug)]
pub(crate) struct LsArgs {
    #[arg(help = "Narrow the list by a query, such as due:today, or a saved filter's name")]
    pub(crate) query: Option<String>,
}

#[derive(Args, Debug)]
pub(crate) struct AgendaArgs {
    #[arg(help = "Narrow the events by a query or saved filter, read the way `ls` reads one")]
    pub(crate) query: Option<String>,
    #[arg(
        long,
        help = "Open the window at a date word, YYYY-MM-DD or YYYY-MM-DDTHH:MM. Default: now"
    )]
    pub(crate) from: Option<String>,
    #[arg(
        long,
        help = "Close the window here, exclusive. Default: 24 hours after it opens"
    )]
    pub(crate) to: Option<String>,
    #[arg(
        long = "max-age",
        value_name = "REMOTE=DURATION",
        value_parser = max_age_arg,
        help = "Refuse when REMOTE last pulled over DURATION ago, such as 15m. Repeatable"
    )]
    pub(crate) max_age: Vec<(RemoteName, Duration)>,
}

#[derive(Args, Debug)]
pub(crate) struct CategoryArgs {
    #[command(subcommand)]
    pub(crate) command: CategoryCommand,
}

#[derive(Subcommand, Debug)]
pub(crate) enum CategoryCommand {
    #[command(about = "List declared categories with their values and whether they are exclusive")]
    List,
}

#[derive(Args, Debug)]
pub(crate) struct FilterArgs {
    #[command(subcommand)]
    pub(crate) command: FilterCommand,
}

#[derive(Subcommand, Debug)]
pub(crate) enum FilterCommand {
    #[command(about = "List saved filters with their queries")]
    List,
}
