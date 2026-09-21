//! `dam category list` and `dam filter list`: what config declares, printed
//! by `dam` so no client owns a second parser for the file `dam` owns.
//!
//! Both read config and nothing else, so `main` answers them before it opens
//! a store: a damaged store cannot fail a listing that never reads one.

use dam_application::Config;

use crate::args::{CategoryCommand, FilterCommand};
use crate::error::CliError;
use crate::output::Report;

pub(crate) fn run_category(config: &Config, command: CategoryCommand) -> Result<Report, CliError> {
    let CategoryCommand::List = command;
    let categories = config.categories.all();
    let rows: Vec<serde_json::Value> = categories
        .iter()
        .map(|c| {
            serde_json::json!({
                "name": c.name,
                "values": c.values,
                "exclusive": c.exclusive,
            })
        })
        .collect();
    let lines: Vec<String> = categories
        .iter()
        .map(|c| {
            let holds = if c.exclusive { "exclusive" } else { "any" };
            format!("{}  {}  {}", c.name, holds, c.values.join(", "))
        })
        .collect();
    Ok(Report {
        human: lines.join("\n"),
        data: serde_json::json!({ "categories": rows }),
    })
}

pub(crate) fn run_filter(config: &Config, command: FilterCommand) -> Result<Report, CliError> {
    let FilterCommand::List = command;
    let rows: Vec<serde_json::Value> = config
        .filters
        .iter()
        .map(|f| serde_json::json!({ "name": f.name, "query": f.query }))
        .collect();
    let lines: Vec<String> = config
        .filters
        .iter()
        .map(|f| format!("{}  {}", f.name, f.query))
        .collect();
    Ok(Report {
        human: lines.join("\n"),
        data: serde_json::json!({ "filters": rows }),
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
