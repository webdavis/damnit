use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub(crate) struct LsArgs {
    pub(crate) query: Option<String>,
}

#[derive(Args, Debug)]
pub(crate) struct AgendaArgs {
    #[arg(help = "A query or saved filter that narrows the events, read the way `ls` reads one")]
    pub(crate) query: Option<String>,
    #[arg(
        long,
        help = "Where the window opens: a date word, YYYY-MM-DD or YYYY-MM-DDTHH:MM. Default: now"
    )]
    pub(crate) from: Option<String>,
    #[arg(
        long,
        help = "Where the window closes, exclusive. Default: 24 hours after it opens"
    )]
    pub(crate) to: Option<String>,
    #[arg(
        long = "max-age",
        value_name = "REMOTE=DURATION",
        help = "Refuse to answer when REMOTE last pulled longer ago than DURATION (such as 90s, 15m \
                or 2h), or never. Repeatable"
    )]
    pub(crate) max_age: Vec<String>,
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
