use dam_application::{Config, FilterConfig};
use serde::Serialize;

mod category_list;

use category_list::run_category;

use crate::args::{CategoryArgs, CategoryCommand, Command, FilterArgs, FilterCommand, ScopeArgs};
use crate::error::CliError;
use crate::output::Report;
use crate::project_scope::Standing;
use crate::view::{Human, View, report};

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

#[derive(Serialize)]
struct FilterList<'a> {
    filters: Vec<FilterRow<'a>>,
}

#[derive(Serialize)]
struct FilterRow<'a> {
    name: &'a str,
    kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    query: Option<&'a str>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    project: bool,
}

impl View for FilterList<'_> {
    fn human(&self) -> Human {
        Human::list(
            self.filters
                .iter()
                .map(|f| {
                    let query = f.query.unwrap_or_default().to_string();
                    let name = match f.project {
                        true => format!("{} (the project's)", f.name),
                        false => f.name.to_string(),
                    };
                    vec![name, f.kind.to_string(), query]
                })
                .collect(),
            "no filters declared",
        )
    }
}

fn run_filter(
    config: &Config,
    command: FilterCommand,
    project_filters: &[FilterConfig],
) -> Result<Report, CliError> {
    let FilterCommand::List = command else {
        unreachable!("filter create and remove open the store")
    };
    let project = project_filters.iter().map(|f| (f, true));
    let personal = config.filters.iter().map(|f| (f, false));
    report(&FilterList {
        filters: project
            .chain(personal)
            .map(|(f, project)| FilterRow {
                name: &f.name,
                kind: match f.rule.runs_a_function() {
                    true => "function",
                    false => "query",
                },
                query: f.rule.query(),
                project,
            })
            .collect(),
    })
}

#[cfg(test)]
mod project_tests;
#[cfg(test)]
mod tests;
