use dam_application::Config;
use serde::Serialize;

use crate::args::{CategoryCommand, Command, FilterArgs, FilterCommand};
use crate::error::CliError;
use crate::output::Report;
use crate::view::{Human, View, report};

pub(crate) fn run(config: &Config, command: Command) -> Result<Report, CliError> {
    match command {
        Command::Category(a) => run_category(config, a.command),
        Command::Filter(a) => run_filter(config, a.command),
        _ => unreachable!("only category and filter read the config alone"),
    }
}

pub(crate) fn only_reads(command: &Command) -> bool {
    matches!(
        command,
        Command::Category(_)
            | Command::Filter(FilterArgs {
                command: FilterCommand::List
            })
    )
}

#[derive(Serialize)]
struct CategoryList<'a> {
    categories: Vec<CategoryRow<'a>>,
}

#[derive(Serialize)]
struct CategoryRow<'a> {
    name: &'a str,
    values: &'a [String],
    exclusive: bool,
}

impl View for CategoryList<'_> {
    fn human(&self) -> Human {
        Human::list(
            self.categories
                .iter()
                .map(|c| {
                    let holds = if c.exclusive { "exclusive" } else { "any" };
                    vec![c.name.to_string(), holds.to_string(), c.values.join(", ")]
                })
                .collect(),
            "no categories declared",
        )
    }
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
}

impl View for FilterList<'_> {
    fn human(&self) -> Human {
        Human::list(
            self.filters
                .iter()
                .map(|f| {
                    let query = f.query.unwrap_or_default().to_string();
                    vec![f.name.to_string(), f.kind.to_string(), query]
                })
                .collect(),
            "no filters declared",
        )
    }
}

fn run_category(config: &Config, command: CategoryCommand) -> Result<Report, CliError> {
    let CategoryCommand::List = command;
    report(&CategoryList {
        categories: config
            .categories
            .all()
            .iter()
            .map(|c| CategoryRow {
                name: &c.name,
                values: &c.values,
                exclusive: c.exclusive,
            })
            .collect(),
    })
}

fn run_filter(config: &Config, command: FilterCommand) -> Result<Report, CliError> {
    let FilterCommand::List = command else {
        unreachable!("filter create and remove open the store")
    };
    report(&FilterList {
        filters: config
            .filters
            .iter()
            .map(|f| FilterRow {
                name: &f.name,
                kind: match f.rule.runs_a_function() {
                    true => "function",
                    false => "query",
                },
                query: f.rule.query(),
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests;
