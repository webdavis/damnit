use dam_application::testing::{FakeSettingsFile, Written, saved, saved_function};
use dam_application::{Config, FilterConfig, SettingValue};
use dam_domain::{FilterTable, TableValue};

use super::filter::{create, remove};
use crate::error::CliError;
use crate::output::{Format, shown};

fn config_with(filters: Vec<FilterConfig>) -> Config {
    Config {
        filters,
        ..Config::default()
    }
}

fn due(text: &str) -> FilterTable {
    FilterTable {
        fields: [("due".into(), TableValue::One(text.into()))].into(),
        any: Vec::new(),
    }
}

fn refusal(result: Result<crate::output::Report, CliError>) -> String {
    match result {
        Err(error) => error.to_string(),
        Ok(_) => panic!("was not refused"),
    }
}

#[test]
fn creating_a_filter_writes_its_table_and_says_so_as_flags() {
    let file = FakeSettingsFile::default();
    let report = create(
        &Config::default(),
        &file,
        "config.lua",
        "soon",
        &due("..today"),
    )
    .unwrap();
    assert_eq!(
        *file.written.borrow(),
        vec![Written::Set(
            "filter.soon".into(),
            SettingValue::Table(due("..today"))
        )]
    );
    assert_eq!(report.human, "created filter soon: --due ..today");
    assert_eq!(
        report.data,
        serde_json::json!({"name": "soon", "table": {"due": "..today"}, "file": "config.lua"})
    );
}

#[test]
fn a_value_a_field_cannot_take_is_refused_and_nothing_is_written() {
    let file = FakeSettingsFile::default();
    let error = refusal(create(
        &Config::default(),
        &file,
        "config.lua",
        "soon",
        &due("soon"),
    ));
    assert!(error.starts_with("--due cannot be \"soon\""), "{error}");
    assert!(file.written.borrow().is_empty());
}

#[test]
fn creating_a_filter_that_exists_is_refused_naming_remove() {
    let file = FakeSettingsFile::default();
    let config = config_with(vec![saved("soon", &[("due", "today")])]);
    assert_eq!(
        refusal(create(
            &config,
            &file,
            "config.lua",
            "soon",
            &due("today+1")
        )),
        "filter soon exists; run dam filter remove soon first, or pick another name"
    );
    assert!(file.written.borrow().is_empty());
}

#[test]
fn removing_a_filter_removes_its_whole_entry() {
    let file = FakeSettingsFile::default();
    let config = config_with(vec![saved("soon", &[("due", "today")])]);
    let report = remove(&config, &file, "config.lua", "soon").unwrap();
    assert_eq!(
        *file.written.borrow(),
        vec![Written::Unset("filter.soon".into())]
    );
    assert_eq!(report.human, "removed filter soon");
    assert_eq!(
        report.data,
        serde_json::json!({"name": "soon", "file": "config.lua"})
    );
}

#[test]
fn removing_a_filter_that_does_not_exist_is_refused() {
    let file = FakeSettingsFile::default();
    assert_eq!(
        refusal(remove(&Config::default(), &file, "config.lua", "soon")),
        "there is no filter soon; dam filter list shows the saved filters"
    );
}

#[test]
fn removing_a_filter_with_a_match_function_is_refused_and_says_where() {
    let file = FakeSettingsFile::default();
    for filter in [
        saved_function("mine", &[]),
        saved_function("mine", &[("due", "today")]),
    ] {
        let config = config_with(vec![filter]);
        assert_eq!(
            refusal(remove(&config, &file, "config.lua", "mine")),
            "filter mine runs a match function, which is written by hand; \
             remove filter.mine from config.lua by hand"
        );
    }
    assert!(file.written.borrow().is_empty());
}

#[test]
fn quiet_prints_nothing_for_a_created_filter() {
    let file = FakeSettingsFile::default();
    let report = create(
        &Config::default(),
        &file,
        "config.lua",
        "soon",
        &due("today"),
    )
    .unwrap();
    assert_eq!(shown(Format::Quiet, false, None, &report).unwrap(), None);
}

#[test]
fn the_answers_match_their_schemas() {
    let file = FakeSettingsFile::default();
    let created = create(
        &Config::default(),
        &file,
        "config.lua",
        "soon",
        &due("today"),
    )
    .unwrap();
    let config = config_with(vec![saved("soon", &[("due", "today")])]);
    let removed = remove(&config, &file, "config.lua", "soon").unwrap();
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
        let request: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../docs/schemas/list.request.schema.json"
        ))
        .unwrap();
        let id = request["$id"].as_str().unwrap().to_string();
        let validator = jsonschema::options()
            .with_resource(id, jsonschema::Resource::from_contents(request).unwrap())
            .build(&schema)
            .unwrap();
        assert!(validator.is_valid(&data), "{data}");
    }
}
