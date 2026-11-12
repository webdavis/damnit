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
