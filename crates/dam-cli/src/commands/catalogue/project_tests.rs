use dam_domain::Categories;

use super::tests::{config, saved};
use super::{run_category, run_filter};
use crate::args::{CategoryCommand, FilterCommand};

#[test]
fn the_projects_filters_come_first_marked_as_the_projects() {
    let mut config = config();
    config.filters = vec![saved("hot", &[("priority", &["4"])], vec![])];
    let project = [saved("hot", &[("priority", &["1"])], vec![])];
    let report = run_filter(&config, FilterCommand::List, &project).unwrap();
    assert_eq!(
        report.human,
        "hot (the project's)  --priority 1\nhot                  --priority 4"
    );
    assert_eq!(
        report.data["filters"],
        serde_json::json!([
            {"name": "hot", "table": {"priority": "1"}, "project": true},
            {"name": "hot", "table": {"priority": "4"}},
        ])
    );
}

fn category(name: &str, values: &[&str]) -> dam_domain::Category {
    dam_domain::Category {
        name: name.into(),
        values: values.iter().map(|v| v.to_string()).collect(),
        exclusive: false,
    }
}

#[test]
fn inside_a_project_the_list_marks_the_projects_yours_and_yours_set_aside() {
    let mut config = config();
    config.categories = Categories::new(vec![
        category("kind", &["bug"]),
        category("place", &["home"]),
    ])
    .unwrap();
    let project = Categories::new(vec![category("triage", &["bug", "feature"])]).unwrap();
    let report = run_category(&config, CategoryCommand::List, Some(&project)).unwrap();
    assert_eq!(
        report.human,
        "triage (the project's)        any  bug, feature\n\
         place                         any  home\n\
         kind (yours, set aside here)  any  bug"
    );
    assert_eq!(
        report.data["categories"],
        serde_json::json!([
            {"name": "triage", "values": ["bug", "feature"], "exclusive": false, "project": true},
            {"name": "place", "values": ["home"], "exclusive": false},
            {"name": "kind", "values": ["bug"], "exclusive": false, "set_aside": true},
        ])
    );
    let schema = include_str!("../../../../../docs/schemas/category-list.schema.json");
    let schema: serde_json::Value = serde_json::from_str(schema).unwrap();
    assert!(
        jsonschema::is_valid(&schema, &report.data),
        "{}",
        report.data
    );
}
