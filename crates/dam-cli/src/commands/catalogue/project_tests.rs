use dam_application::{FilterConfig, FilterRule};

use super::run_filter;
use super::tests::config;
use crate::args::FilterCommand;

fn query(name: &str, text: &str) -> FilterConfig {
    FilterConfig {
        name: name.into(),
        rule: FilterRule::Query(text.into()),
    }
}

#[test]
fn the_projects_filters_come_first_marked_as_the_projects() {
    let mut config = config();
    config.filters = vec![query("hot", "p4")];
    let project = [query("hot", "p1")];
    let report = run_filter(&config, FilterCommand::List, &project).unwrap();
    assert_eq!(
        report.human,
        "hot (the project's)  query  p1\nhot                  query  p4"
    );
    assert_eq!(
        report.data["filters"],
        serde_json::json!([
            {"name": "hot", "kind": "query", "query": "p1", "project": true},
            {"name": "hot", "kind": "query", "query": "p4"},
        ])
    );
}
