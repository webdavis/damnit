use dam_application::Config;

mod category_list;
mod filter_list;

use category_list::run_category;
use filter_list::run_filter;

use crate::args::{CategoryArgs, CategoryCommand, Command, FilterArgs, FilterCommand, ScopeArgs};
use crate::error::CliError;
use crate::output::Report;
use crate::project_scope::Standing;

pub(crate) fn run(
    config: &Config,
    command: Command,
    standing: &Standing,
) -> Result<Report, CliError> {
    match command {
        Command::Category(a) => {
            let project = standing.rules(&ScopeArgs::default());
            run_category(config, a.command, project.map(|rules| &rules.categories))
        }
        Command::Filter(a) => run_filter(config, a.command, standing.filters()),
        _ => unreachable!("only category and filter read the config alone"),
    }
}

pub(crate) fn only_reads(command: &Command) -> bool {
    matches!(
        command,
        Command::Category(CategoryArgs {
            command: CategoryCommand::List
        }) | Command::Filter(FilterArgs {
            command: FilterCommand::List
        })
    )
}

#[cfg(test)]
mod project_tests;
#[cfg(test)]
mod tests;
