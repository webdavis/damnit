use dam_application::{Config, FilterConfig};
use serde::Serialize;
use serde_json::Value;

use crate::args::FilterCommand;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{Human, View, flags_of, report, table_json};

#[derive(Serialize)]
struct FilterList {
    filters: Vec<FilterRow>,
}

#[derive(Serialize)]
struct FilterRow {
    name: String,
    table: Value,
    #[serde(rename = "match", skip_serializing_if = "std::ops::Not::not")]
    runs_a_function: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    project: bool,
    #[serde(skip)]
    selection: String,
}

impl View for FilterList {
    fn human(&self) -> Human {
        Human::list(
            self.filters
                .iter()
                .map(|f| {
                    let name = match f.project {
                        true => format!("{} (the project's)", f.name),
                        false => f.name.clone(),
                    };
                    let mut row = vec![name, f.selection.clone()];
                    if f.runs_a_function {
                        row.push("and its match function".into());
                    }
                    row
                })
                .collect(),
            "no filters declared",
        )
    }
}

pub(super) fn run_filter(
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
        filters: project.chain(personal).map(|(f, p)| row(f, p)).collect(),
    })
}

fn row(filter: &FilterConfig, project: bool) -> FilterRow {
    let table = table_json(&filter.table);
    let selection = match flags_of(&filter.table) {
        Some(flags) if flags.is_empty() => "-".to_string(),
        Some(flags) => flags,
        None => table.to_string(),
    };
    FilterRow {
        name: filter.name.clone(),
        table,
        runs_a_function: filter.runs_a_function,
        project,
        selection,
    }
}
