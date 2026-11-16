use clap::{Args, Subcommand};
use dam_domain::Path;

use super::ScopeArgs;

#[derive(Args, Debug)]
pub(crate) struct DayArgs {
    #[command(subcommand)]
    pub(crate) command: DayCommand,
}

#[derive(Subcommand, Debug)]
pub(crate) enum DayCommand {
    #[command(about = "Show the record of one day")]
    Show {
        #[arg(help = "Pick the day: a date word, today+N or YYYY-MM-DD. Default: today")]
        day: Option<String>,
    },
    #[command(about = "Replace a day's record with the JSON or TOON document read from stdin")]
    Set {
        #[arg(skip)]
        document: Option<String>,
    },
    #[command(about = "List the recorded days, newest first, with when each was closed")]
    List,
}

#[derive(Args, Debug)]
pub(crate) struct RemindersArgs {
    #[arg(
        long,
        help = "Open the window at a day, instant or anchor: 2026-11-20, 2026-11-20T09:00, today+1, week, now+2h. Default: today"
    )]
    pub(crate) from: Option<String>,
    #[arg(
        long,
        help = "Close the window here; a day or period counts whole, an instant is not included. Default: now"
    )]
    pub(crate) to: Option<String>,
    #[command(flatten)]
    pub(crate) scope: ScopeArgs,
    #[arg(skip)]
    pub(crate) place: Path,
}
