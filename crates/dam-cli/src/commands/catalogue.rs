use dam_application::Config;
use serde::Serialize;

use crate::args::{CategoryCommand, FilterCommand};
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
}

impl View for CategoryList<'_> {
    fn human(&self) -> Human {
        Human::Table(
            self.categories
                .iter()
                .map(|c| {
                    let holds = if c.exclusive { "exclusive" } else { "any" };
                    vec![c.name.to_string(), holds.to_string(), c.values.join(", ")]
                })
                .collect(),
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
        Human::Table(
            self.filters
                .iter()
                .map(|f| vec![f.name.to_string(), f.query.to_string()])
                .collect(),
        )
    }
}

pub(crate) fn run_category(config: &Config, command: CategoryCommand) -> Result<Report, CliError> {
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

pub(crate) fn run_filter(config: &Config, command: FilterCommand) -> Result<Report, CliError> {
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
mod tests {
    use super::*;
    use dam_application::FilterConfig;
    use dam_domain::{Categories, Category};

    fn config() -> Config {
        Config {
            done_interactive: false,
            remotes: vec![],
            categories: Categories::default(),
            filters: vec![],
        }
    }

    #[test]
    fn a_category_line_names_the_values_and_whether_it_holds_one() {
        let mut config = config();
        config.categories = Categories::new(vec![
            Category {
                name: "effort".into(),
                values: vec!["light".into(), "deep".into()],
                exclusive: true,
            },
            Category {
                name: "context".into(),
                values: vec!["home".into()],
                exclusive: false,
            },
        ])
        .unwrap();
        let report = run_category(&config, CategoryCommand::List).unwrap();
        assert_eq!(
            report.human,
            "effort  exclusive  light, deep\ncontext  any  home"
        );
        assert_eq!(
            report.data["categories"][0],
            serde_json::json!({"name": "effort", "values": ["light", "deep"], "exclusive": true})
        );
    }

    #[test]
    fn a_filter_line_is_its_name_and_its_query() {
        let mut config = config();
        config.filters = vec![FilterConfig {
            name: "today".into(),
            query: "due:today | overdue".into(),
        }];
        let report = run_filter(&config, FilterCommand::List).unwrap();
        assert_eq!(report.human, "today  due:today | overdue");
        assert_eq!(
            report.data["filters"][0],
            serde_json::json!({"name": "today", "query": "due:today | overdue"})
        );
    }

    #[test]
    fn nothing_declared_is_an_empty_list_rather_than_a_failure() {
        let config = config();
        assert_eq!(
            run_category(&config, CategoryCommand::List).unwrap().data,
            serde_json::json!({"categories": []})
        );
        assert_eq!(
            run_filter(&config, FilterCommand::List).unwrap().data,
            serde_json::json!({"filters": []})
        );
    }
}
