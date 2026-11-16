use super::*;
use dam_application::FilterConfig;
use dam_domain::{Categories, Category, FilterTable, TableValue};

pub(super) fn config() -> Config {
    Config {
        owner: None,
        done_interactive: None,
        progress: None,
        confirm_above: None,
        remotes: vec![],
        categories: Categories::default(),
        project_categories: None,
        categories_writing_exclusive: Default::default(),
        filters: vec![],
        hooks: vec![],
        task_pulled: None,
        project_hooks: None,
        tokens: Default::default(),
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
    let report = run_category(&config, CategoryCommand::List, None).unwrap();
    assert_eq!(
        report.human,
        "effort   exclusive  light, deep\ncontext  any        home"
    );
    assert_eq!(
        report.data["categories"][0],
        serde_json::json!({"name": "effort", "values": ["light", "deep"], "exclusive": true})
    );
}

pub(super) fn saved(name: &str, fields: &[(&str, &[&str])], any: Vec<FilterTable>) -> FilterConfig {
    let value = |texts: &[&str]| match texts {
        [one] => TableValue::One(one.to_string()),
        many => TableValue::Many(many.iter().map(|t| t.to_string()).collect()),
    };
    FilterConfig {
        name: name.into(),
        table: FilterTable {
            fields: fields
                .iter()
                .map(|(k, v)| (k.to_string(), value(v)))
                .collect(),
            any,
        },
        runs_a_function: false,
    }
}

#[test]
fn a_filter_line_is_its_name_and_the_flags_that_select_it() {
    let mut config = config();
    let urgent = saved("", &[("labels", &["urgent"])], vec![]).table;
    config.filters = vec![saved(
        "today",
        &[("due", &["..today"]), ("labels", &["bug", "not:waiting"])],
        vec![urgent],
    )];
    let report = run_filter(&config, FilterCommand::List, &[]).unwrap();
    assert_eq!(
        report.human,
        "today  --due ..today --labels bug --labels not:waiting --any labels=urgent"
    );
    assert_eq!(
        report.data["filters"][0],
        serde_json::json!({"name": "today", "table": {
            "due": "..today", "labels": ["bug", "not:waiting"], "any": [{"labels": "urgent"}]
        }})
    );
}

#[test]
fn a_table_the_flags_cannot_say_is_shown_as_its_table() {
    let mut config = config();
    let both = saved("", &[("labels", &["a"]), ("due", &["today"])], vec![]).table;
    config.filters = vec![saved("odd", &[], vec![both])];
    let report = run_filter(&config, FilterCommand::List, &[]).unwrap();
    assert_eq!(
        report.human,
        r#"odd  {"any":[{"due":"today","labels":"a"}]}"#
    );
}

#[test]
fn a_function_filter_shows_its_fields_but_never_its_source() {
    let mut config = config();
    let mut slipping = saved("slipping", &[], vec![]);
    slipping.runs_a_function = true;
    let mut stuck = saved("stuck", &[("path", &["work/"])], vec![]);
    stuck.runs_a_function = true;
    config.filters = vec![slipping, stuck];
    let report = run_filter(&config, FilterCommand::List, &[]).unwrap();
    assert_eq!(
        report.human,
        "slipping  -             and its match function
stuck     --path work/  and its match function"
    );
    assert_eq!(
        report.data["filters"],
        serde_json::json!([
            {"name": "slipping", "table": {}, "match": true},
            {"name": "stuck", "table": {"path": "work/"}, "match": true},
        ])
    );
}

#[test]
fn nothing_declared_is_an_empty_list_rather_than_a_failure() {
    let config = config();
    assert_eq!(
        run_category(&config, CategoryCommand::List, None)
            .unwrap()
            .data,
        serde_json::json!({"categories": []})
    );
    assert_eq!(
        run_filter(&config, FilterCommand::List, &[]).unwrap().data,
        serde_json::json!({"filters": []})
    );
    assert_eq!(
        run_category(&config, CategoryCommand::List, None)
            .unwrap()
            .human,
        "no categories declared"
    );
    assert_eq!(
        run_filter(&config, FilterCommand::List, &[]).unwrap().human,
        "no filters declared"
    );
}
