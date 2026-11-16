use dam_application::{ConfigError, SelectError};
use dam_domain::{DefaultsError, FilterError, Problem};

use super::text;

fn table_error(field: &str, value: &str, problem: Problem) -> ConfigError {
    ConfigError::FilterTable {
        file: "config.lua".into(),
        key: "filter.today".into(),
        error: Box::new(SelectError::Value(FilterError {
            field: field.into(),
            value: value.into(),
            problem,
        })),
    }
}

#[test]
fn query_text_shows_the_table_to_write_or_the_general_form() {
    let translated = ConfigError::QueryText {
        file: "config.lua".into(),
        was: "filter.today.query".into(),
        key: "filter.today".into(),
        table: Some(r#"{ due = "today" }"#.into()),
    };
    assert_eq!(
        text(translated),
        r#"config: filter.today.query in config.lua is query text, which dam no longer reads; write filter.today = { due = "today" } instead"#
    );
    let general = ConfigError::QueryText {
        file: "/work/app/.dam.lua".into(),
        was: "ls".into(),
        key: "list".into(),
        table: None,
    };
    assert_eq!(
        text(general),
        r#"config: ls in /work/app/.dam.lua is query text, which dam no longer reads; write list as a table of dam list flags instead, such as list = { due = "..today", labels = "bug" }"#
    );
}

#[test]
fn a_field_of_a_filter_is_named_with_its_fix() {
    let closest = Problem::UnknownField {
        closest: Some("labels"),
    };
    assert_eq!(
        text(table_error("lables", "x", closest)),
        "config: filter.today.lables in config.lua names no field; the closest is labels"
    );
    assert_eq!(
        text(table_error("due", "soon", Problem::BadValue)),
        "config: filter.today.due in config.lua cannot be \"soon\"; it takes 2026-11-20, \
         2026-11-20T09:00, 2026-W47, 2026-11, 2026, today, today+3, week, week+1, month-1, \
         year, now+2h, A..B, ..B, A.., on:thu, on:1tue, on:-1fri, on:15, on:-1, on:FILTER, \
         not:DATE, none"
    );
    let two = Problem::TwoAtOnce {
        fix: Some("1,2".into()),
    };
    assert_eq!(
        text(table_error("priority", "2", two)),
        "config: filter.today.priority in config.lua cannot hold two values at once; \
         write priority = \"1,2\" for either"
    );
}

#[test]
fn fields_that_contradict_are_named() {
    let error = ConfigError::FilterTable {
        file: "config.lua".into(),
        key: "filter.today".into(),
        error: Box::new(SelectError::Defaults(DefaultsError::BothKinds {
            task_field: "due",
            event_field: "start",
        })),
    };
    assert_eq!(
        text(error),
        "config: filter.today in config.lua names due, a task field, and start, an event field; \
         put them under any for either"
    );
}

#[test]
fn a_filter_that_reaches_itself_names_the_chain() {
    let error = ConfigError::FilterReachesItself {
        file: "config.lua".into(),
        chain: vec!["a".into(), "b".into(), "a".into()],
    };
    assert_eq!(
        text(error),
        "config: filter a in config.lua reaches itself (a -> b -> a); \
         remove one of those filter, in: or on: values"
    );
}
