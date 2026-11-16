use std::time::Duration;

use clap::{Args, Subcommand};
use dam_application::{FilterConfig, RemoteName};
use dam_domain::Path;

use super::ScopeArgs;
use super::selection::SelectArgs;

use super::values::max_age_arg;

#[derive(Args, Debug, Default)]
pub(crate) struct ListArgs {
    #[command(flatten)]
    pub(crate) selection: SelectArgs,
    #[command(flatten)]
    pub(crate) scope: ScopeArgs,
    #[arg(skip)]
    pub(crate) place: Path,
    #[arg(skip)]
    pub(crate) project_default: Option<String>,
    #[arg(skip)]
    pub(crate) project_filters: Vec<FilterConfig>,
}

#[derive(Args, Debug, Default)]
pub(crate) struct AgendaArgs {
    #[arg(help = "Narrow the events by a query, such as due:today, or a saved filter's name")]
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
    #[command(flatten)]
    pub(crate) scope: ScopeArgs,
    #[arg(skip)]
    pub(crate) place: Path,
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
    #[command(about = "Declare a category in config.lua, or in .dam.lua inside a project")]
    Create {
        name: String,
        #[arg(long, help = "The labels of the category, separated by commas")]
        values: String,
        #[arg(long, help = "Let a task hold at most one of its labels")]
        exclusive: bool,
        #[arg(long, help = "Write config.lua inside a project, not .dam.lua")]
        global: bool,
    },
    #[command(
        about = "Remove a category from config.lua, or from .dam.lua inside a project; labels on tasks stay as free labels"
    )]
    Remove {
        name: String,
        #[arg(long, help = "Write config.lua inside a project, not .dam.lua")]
        global: bool,
    },
    #[command(about = "Rename labels on every task and event that carries them, as one change")]
    Migrate {
        #[arg(
            long,
            value_name = "OLD=NEW",
            help = "Rename the label OLD to NEW. Repeatable"
        )]
        map: Vec<String>,
        #[arg(long, help = "Show the changes and write nothing")]
        preview: bool,
        #[arg(
            long,
            value_name = "N",
            help = "Refuse, changing nothing, unless exactly N objects carry an old label"
        )]
        expect: Option<usize>,
        #[arg(
            long,
            help = "Also rewrite each category's values and saved filter's query in config.lua; \
                    dam cannot see inside a match function, so that filter is left to edit by hand"
        )]
        update_config: bool,
    },
}

#[derive(Args, Debug)]
pub(crate) struct FilterArgs {
    #[command(subcommand)]
    pub(crate) command: FilterCommand,
}

#[derive(Subcommand, Debug)]
pub(crate) enum FilterCommand {
    #[command(about = "List saved filters with their kinds and queries")]
    List,
    #[command(about = "Save a filter in config.lua, or in .dam.lua inside a project")]
    Create {
        name: String,
        #[arg(long, help = "The query the filter runs, such as due:today")]
        query: String,
        #[arg(long, help = "Write config.lua inside a project, not .dam.lua")]
        global: bool,
    },
    #[command(about = "Remove a saved filter from config.lua, or from .dam.lua inside a project")]
    Remove {
        name: String,
        #[arg(long, help = "Write config.lua inside a project, not .dam.lua")]
        global: bool,
    },
}
