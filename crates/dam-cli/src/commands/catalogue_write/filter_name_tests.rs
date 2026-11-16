use dam_application::Config;
use dam_application::testing::{FakeSettingsFile, saved};
use dam_domain::{FilterTable, TableValue};

use super::filter::create;
use crate::error::CliError;

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
fn a_name_a_value_already_means_is_refused() {
    let file = FakeSettingsFile::default();
    let made = |name: &str| {
        refusal(create(
            &Config::default(),
            &file,
            "config.lua",
            name,
            &due("today"),
        ))
    };
    assert_eq!(made(""), "a filter needs a name");
    assert_eq!(
        made("none"),
        "a filter cannot be named none, which means an empty field; pick another name"
    );
    assert_eq!(
        made("thu"),
        "a filter cannot be named thu, which on: reads as a day; pick another name"
    );
    assert_eq!(
        made("in:x"),
        "a filter's name cannot start with in:, which a value reads; pick another name"
    );
    assert_eq!(
        made("a,b"),
        "a filter's name cannot hold a comma, which separates names in a list; pick another name"
    );
    assert!(file.written.borrow().is_empty());
}

#[test]
fn a_filter_that_would_reach_itself_is_refused_naming_the_chain() {
    let file = FakeSettingsFile::default();
    let config = Config {
        filters: vec![saved("week", &[("filter", "soon")])],
        ..Config::default()
    };
    let table = FilterTable {
        fields: [("parent".into(), TableValue::One("in:week".into()))].into(),
        any: Vec::new(),
    };
    assert_eq!(
        refusal(create(&config, &file, "config.lua", "soon", &table)),
        "filter soon reaches itself (soon -> week -> soon); remove one of those filter, in: or on: values"
    );
    assert!(file.written.borrow().is_empty());
}
