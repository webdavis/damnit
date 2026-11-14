use dam_application::Config;
use serde::Serialize;

use crate::args::{CategoryCommand, Command, FilterCommand};
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
    query: &'a str,
}

impl View for FilterList<'_> {
    fn human(&self) -> Human {
        Human::list(
            self.filters
                .iter()
                .map(|f| vec![f.name.to_string(), f.query.to_string()])
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
    let FilterCommand::List = command;
    report(&FilterList {
        filters: config
            .filters
            .iter()
            .map(|f| FilterRow {
                name: &f.name,
                query: &f.query,
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests;
