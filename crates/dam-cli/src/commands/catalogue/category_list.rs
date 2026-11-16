use dam_application::Config;
use dam_domain::{Categories, Category};
use serde::Serialize;

use crate::args::CategoryCommand;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{Human, View, report};

#[derive(Serialize)]
struct CategoryList<'a> {
    categories: Vec<CategoryRow<'a>>,
}

#[derive(Serialize)]
struct CategoryRow<'a> {
    name: &'a str,
    values: &'a [String],
    exclusive: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    project: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    set_aside: bool,
}

impl View for CategoryList<'_> {
    fn human(&self) -> Human {
        Human::list(
            self.categories
                .iter()
                .map(|c| {
                    let holds = if c.exclusive { "exclusive" } else { "any" };
                    let name = match (c.project, c.set_aside) {
                        (true, _) => format!("{} (the project's)", c.name),
                        (_, true) => format!("{} (yours, set aside here)", c.name),
                        _ => c.name.to_string(),
                    };
                    vec![name, holds.to_string(), c.values.join(", ")]
                })
                .collect(),
            "no categories declared",
        )
    }
}

fn row(c: &Category, project: bool, set_aside: bool) -> CategoryRow<'_> {
    CategoryRow {
        name: &c.name,
        values: &c.values,
        exclusive: c.exclusive,
        project,
        set_aside,
    }
}

pub(super) fn run_category(
    config: &Config,
    command: CategoryCommand,
    project: Option<&Categories>,
) -> Result<Report, CliError> {
    let CategoryCommand::List = command else {
        unreachable!("category create and remove open the store")
    };
    let none = Categories::default();
    let project = project.unwrap_or(&none);
    let aside: Vec<&str> = config
        .categories
        .set_aside_by(project)
        .into_iter()
        .map(|(yours, _)| yours)
        .collect();
    let (set_aside, in_force): (Vec<_>, Vec<_>) = config
        .categories
        .all()
        .iter()
        .partition(|c| aside.contains(&c.name.as_str()));
    let rows = project.all().iter().map(|c| row(c, true, false));
    let rows = rows.chain(in_force.into_iter().map(|c| row(c, false, false)));
    let rows = rows.chain(set_aside.into_iter().map(|c| row(c, false, true)));
    report(&CategoryList {
        categories: rows.collect(),
    })
}
