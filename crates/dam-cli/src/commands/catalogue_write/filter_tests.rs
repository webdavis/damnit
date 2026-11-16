use dam_application::testing::{FakeSettingsFile, Written};
use dam_application::{Config, FilterConfig, FilterRule, SettingValue};

use super::filter::{create, remove};
use crate::error::CliError;
use crate::output::{Format, shown};

fn config_with(filters: Vec<FilterConfig>) -> Config {
    Config {
        filters,
        ..Config::default()
    }
}

fn filter(name: &str, rule: FilterRule) -> FilterConfig {
    FilterConfig {
        name: name.into(),
        rule,
    }
}

fn refusal(result: Result<crate::output::Report, CliError>) -> String {
    match result {
        Err(error) => error.to_string(),
        Ok(_) => panic!("was not refused"),
    }
}

#[test]
fn creating_a_filter_writes_its_query_and_says_so() {
    let file = FakeSettingsFile::default();
    let report = create(&Config::default(), &file, "soon", "due:today").unwrap();
    assert_eq!(
        *file.written.borrow(),
        vec![Written::Set(
            "filter.soon.query".into(),
            SettingValue::Text("due:today".into())
        )]
    );
    assert_eq!(report.human, "created filter soon: due:today");
    assert_eq!(
        report.data,
        serde_json::json!({"name": "soon", "query": "due:today"})
    );
}

#[test]
fn a_query_that_does_not_parse_is_refused_and_nothing_is_written() {
    let file = FakeSettingsFile::default();
    let error = create(&Config::default(), &file, "soon", "(due:today").unwrap_err();
    assert!(matches!(
        error,
        CliError::UseCase(dam_application::UseCaseError::Query(_))
    ));
    assert!(file.written.borrow().is_empty());
}

#[test]
fn creating_a_filter_that_exists_is_refused_naming_config_set() {
    let file = FakeSettingsFile::default();
    let config = config_with(vec![filter("soon", FilterRule::Query("due:today".into()))]);
    assert_eq!(
        refusal(create(&config, &file, "soon", "due:tomorrow")),
        "filter soon exists; change its query with dam config set filter.soon.query '<query>'"
    );
    assert!(file.written.borrow().is_empty());
}

#[test]
fn a_filter_without_a_name_is_refused() {
    let file = FakeSettingsFile::default();
    assert_eq!(
        refusal(create(&Config::default(), &file, "", "due:today")),
        "a filter needs a name"
    );
}

#[test]
fn removing_a_filter_removes_its_whole_entry() {
    let file = FakeSettingsFile::default();
    let config = config_with(vec![filter("soon", FilterRule::Query("due:today".into()))]);
    let report = remove(&config, &file, "soon").unwrap();
    assert_eq!(
        *file.written.borrow(),
        vec![Written::Unset("filter.soon".into())]
    );
    assert_eq!(report.human, "removed filter soon");
    assert_eq!(report.data, serde_json::json!({"name": "soon"}));
}

#[test]
fn removing_a_filter_that_does_not_exist_is_refused() {
    let file = FakeSettingsFile::default();
    assert_eq!(
        refusal(remove(&Config::default(), &file, "soon")),
        "there is no filter soon; dam filter list shows the saved filters"
    );
}

#[test]
fn removing_a_filter_with_a_match_function_is_refused_and_says_where() {
    let file = FakeSettingsFile::default();
    for rule in [
        FilterRule::Function,
        FilterRule::QueryThenFunction("due:today".into()),
    ] {
        let config = config_with(vec![filter("mine", rule)]);
        assert_eq!(
            refusal(remove(&config, &file, "mine")),
            "filter mine runs a match function, which is written by hand; \
             remove filter.mine from config.lua by hand"
        );
    }
    assert!(file.written.borrow().is_empty());
}

#[test]
fn quiet_prints_nothing_for_a_created_filter() {
    let file = FakeSettingsFile::default();
    let report = create(&Config::default(), &file, "soon", "due:today").unwrap();
    assert_eq!(shown(Format::Quiet, false, None, &report).unwrap(), None);
}

#[test]
fn the_answers_match_their_schemas() {
    let file = FakeSettingsFile::default();
    let created = create(&Config::default(), &file, "soon", "due:today").unwrap();
    let config = config_with(vec![filter("soon", FilterRule::Query("due:today".into()))]);
    let removed = remove(&config, &file, "soon").unwrap();
    for (data, schema) in [
        (
            created.data,
            include_str!("../../../../../docs/schemas/filter-create.schema.json"),
        ),
        (
            removed.data,
            include_str!("../../../../../docs/schemas/filter-remove.schema.json"),
        ),
    ] {
        let schema: serde_json::Value = serde_json::from_str(schema).unwrap();
        assert!(jsonschema::is_valid(&schema, &data), "{data}");
    }
}
