use super::*;
use dam_application::{FilterConfig, FilterRule};
use dam_domain::{Categories, Category};

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

#[test]
fn a_filter_line_is_its_name_its_kind_and_its_query() {
    let mut config = config();
    config.filters = vec![FilterConfig {
        name: "today".into(),
        rule: FilterRule::Query("due:today | overdue".into()),
    }];
    let report = run_filter(&config, FilterCommand::List, &[]).unwrap();
    assert_eq!(report.human, "today  query  due:today | overdue");
    assert_eq!(
        report.data["filters"][0],
        serde_json::json!({"name": "today", "kind": "query", "query": "due:today | overdue"})
    );
}

#[test]
fn a_function_filter_shows_its_kind_and_any_query_but_never_its_source() {
    let mut config = config();
    config.filters = vec![
        FilterConfig {
            name: "slipping".into(),
            rule: FilterRule::Function,
        },
        FilterConfig {
            name: "stuck".into(),
            rule: FilterRule::QueryThenFunction("path:work/".into()),
        },
    ];
    let report = run_filter(&config, FilterCommand::List, &[]).unwrap();
    assert_eq!(
        report.human,
        "slipping  function  -\nstuck     function  path:work/"
    );
    assert_eq!(
        report.data["filters"],
        serde_json::json!([
            {"name": "slipping", "kind": "function"},
            {"name": "stuck", "kind": "function", "query": "path:work/"},
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
