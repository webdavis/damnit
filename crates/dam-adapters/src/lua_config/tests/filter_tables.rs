use dam_application::{ConfigError, Expected, FilterConfig, Misread, SelectError};
use dam_domain::{FilterTable, Problem, TableValue};

use super::support::{config, in_table};

fn read(filter: &str) -> FilterConfig {
    let text = format!("return {{ filter = {{ slipping = {filter} }} }}");
    config(&text)
        .load()
        .unwrap()
        .filter("slipping")
        .unwrap()
        .clone()
}

fn refusal(filter: &str) -> ConfigError {
    config(&format!("return {{ filter = {{ slipping = {filter} }} }}")).refusal()
}

fn one(text: &str) -> TableValue {
    TableValue::One(text.into())
}

fn table(fields: &[(&str, TableValue)], any: Vec<FilterTable>) -> FilterTable {
    FilterTable {
        fields: fields
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect(),
        any,
    }
}

#[test]
fn a_filter_reads_each_field_as_the_text_typed_after_its_flag() {
    assert_eq!(
        read("{ due = '..today' }").table,
        table(&[("due", one("..today"))], vec![])
    );
    assert_eq!(
        read("{ kind = 'event', subject = 'all hands' }").table,
        table(
            &[("kind", one("event")), ("subject", one("all hands"))],
            vec![]
        )
    );
    assert_eq!(
        read("{ priority = 1, overdue = true, ['focus.day'] = 'today' }").table,
        table(
            &[
                ("priority", one("1")),
                ("overdue", one("true")),
                ("focus.day", one("today"))
            ],
            vec![]
        )
    );
}

#[test]
fn a_list_repeats_the_flag_and_any_holds_tables() {
    assert_eq!(
        read("{ labels = { 'bug', 'not:waiting' } }").table,
        table(
            &[(
                "labels",
                TableValue::Many(vec!["bug".into(), "not:waiting".into()])
            )],
            vec![]
        )
    );
    assert_eq!(
        read("{ any = { { labels = 'urgent' }, { due = '..today' } } }").table,
        table(
            &[],
            vec![
                table(&[("labels", one("urgent"))], vec![]),
                table(&[("due", one("..today"))], vec![])
            ]
        )
    );
}

#[test]
fn a_match_function_runs_alone_or_beside_the_fields() {
    let alone = read("{ match = function(task, dam) return true end }");
    assert!(alone.runs_a_function && alone.table == FilterTable::default());
    let beside = read("{ path = 'work/', match = function() return true end }");
    assert!(beside.runs_a_function);
    assert_eq!(beside.table, table(&[("path", one("work/"))], vec![]));
}

#[test]
fn an_old_query_is_refused_with_the_table_to_write() {
    assert_eq!(
        refusal("{ query = 'due:today' }"),
        ConfigError::QueryText {
            file: "config.lua".into(),
            was: "filter.slipping.query".into(),
            key: "filter.slipping".into(),
            table: Some(r#"{ done = "any", due = "today" }"#.into()),
        }
    );
    assert_eq!(
        refusal("{ query = '!overdue' }"),
        ConfigError::QueryText {
            file: "config.lua".into(),
            was: "filter.slipping.query".into(),
            key: "filter.slipping".into(),
            table: None,
        }
    );
}

#[test]
fn a_field_dam_does_not_know_names_the_closest() {
    let ConfigError::FilterTable { key, error, .. } = refusal("{ lables = 'x' }") else {
        panic!("an unknown field was read");
    };
    assert_eq!(key, "filter.slipping");
    let SelectError::Value(error) = *error else {
        panic!("not a value error");
    };
    assert_eq!(
        error.problem,
        Problem::UnknownField {
            closest: Some("labels")
        }
    );
}

#[test]
fn a_value_a_field_cannot_take_is_refused() {
    let ConfigError::FilterTable { error, .. } = refusal("{ due = 'soon' }") else {
        panic!("a bad value was read");
    };
    let SelectError::Value(error) = *error else {
        panic!("not a value error");
    };
    assert_eq!(
        (error.field.as_str(), error.problem),
        ("due", Problem::BadValue)
    );
}

#[test]
fn a_value_that_is_not_text_a_number_or_true_or_false_is_refused() {
    assert_eq!(
        refusal("{ due = { { 'today' } } }"),
        Misread::WrongType {
            key: "due".into(),
            at: in_table("filter.slipping", "config.lua"),
            expected: Expected::FilterValue,
        }
        .into()
    );
    assert_eq!(
        refusal("{ any = { 'x' } }"),
        Misread::WrongType {
            key: "any".into(),
            at: in_table("filter.slipping", "config.lua"),
            expected: Expected::ListOfTables,
        }
        .into()
    );
}

#[test]
fn a_match_that_is_not_a_function_is_refused_by_name() {
    assert_eq!(
        refusal("{ match = 'due:today' }"),
        Misread::WrongType {
            key: "match".into(),
            at: in_table("filter.slipping", "config.lua"),
            expected: Expected::Function,
        }
        .into()
    );
}

#[test]
fn a_filter_that_reaches_itself_is_refused_naming_the_chain() {
    let text = "return { filter = { a = { filter = 'b' }, b = { parent = 'in:c' }, \
                c = { due = 'on:a' }, d = { filter = 'a' } } }";
    assert_eq!(
        config(text).refusal(),
        ConfigError::FilterReachesItself {
            file: "config.lua".into(),
            chain: vec!["a".into(), "b".into(), "c".into(), "a".into()],
        }
    );
}
