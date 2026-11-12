use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub(crate) struct DayArgs {
    #[command(subcommand)]
    pub(crate) command: DayCommand,
}

#[derive(Subcommand, Debug)]
pub(crate) enum DayCommand {
    #[command(about = "Show the record of one day")]
    Show {
        #[arg(help = "Pick the day: a date word or YYYY-MM-DD. Default: today")]
        day: Option<String>,
    },
    #[command(about = "Replace a day's record with the JSON or TOON document read from stdin")]
    Set,
    #[command(about = "List the recorded days, newest first, with when each was closed")]
    List,
}

#[derive(Args, Debug)]
pub(crate) struct RemindersArgs {
    #[arg(
        long,
        help = "Open the window at a date word, YYYY-MM-DD or YYYY-MM-DDTHH:MM. Default: today"
    )]
    pub(crate) from: Option<String>,
    #[arg(long, help = "Close the window here, exclusive. Default: now")]
    pub(crate) to: Option<String>,
}
