use super::*;
use dam_application::FilterConfig;
use dam_domain::{Categories, Category};

fn config() -> Config {
    Config {
        owner: None,
        done_interactive: false,
        remotes: vec![],
        categories: Categories::default(),
        filters: vec![],
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
    let report = run_category(&config, CategoryCommand::List).unwrap();
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
    assert_eq!(
        run_category(&config, CategoryCommand::List).unwrap().human,
        "no categories declared"
    );
    assert_eq!(
        run_filter(&config, FilterCommand::List).unwrap().human,
        "no filters declared"
    );
}
