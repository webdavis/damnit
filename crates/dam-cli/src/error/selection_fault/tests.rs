use dam_application::SelectError;
use dam_domain::{DefaultsError, Filter, FilterTable, TableValue};

use super::Fault;
use crate::error::CliError;

fn table(fields: &[(&str, &str)], any: &[(&str, &str)]) -> FilterTable {
    let one = |pairs: &[(&str, &str)]| {
        let mut t = FilterTable::default();
        for (k, v) in pairs {
            let value = match t.fields.remove(*k) {
                None => TableValue::One(v.to_string()),
                Some(TableValue::One(was)) => TableValue::Many(vec![was, v.to_string()]),
                Some(TableValue::Many(mut was)) => {
                    was.push(v.to_string());
                    TableValue::Many(was)
                }
            };
            t.fields.insert(k.to_string(), value);
        }
        t
    };
    let mut t = one(fields);
    t.any = any.iter().map(|pair| one(&[*pair])).collect();
    t
}

fn read(typed: FilterTable) -> CliError {
    let error = Filter::read(&typed).unwrap_err();
    CliError::Selection(Box::new(Fault {
        error: SelectError::Value(error),
        typed,
    }))
}

fn defaults(error: DefaultsError, typed: FilterTable) -> CliError {
    CliError::Selection(Box::new(Fault {
        error: SelectError::Defaults(error),
        typed,
    }))
}

#[test]
fn a_bad_value_names_the_flag_the_value_the_shapes_and_a_command_and_exits_2() {
    let e = read(table(&[("labels", "bug"), ("due", "tomorow")], &[]));
    assert_eq!(e.exit_code(), 2);
    let text = e.to_string();
    assert!(
        text.starts_with("--due cannot be \"tomorow\"; it takes 2026-11-20,"),
        "{text}"
    );
    assert!(text.contains("on:thu"), "{text}");
    assert!(
        text.ends_with("; run dam list --due today --labels bug"),
        "{text}"
    );
}

#[test]
fn the_same_fault_is_data_under_output_json() {
    let doc = read(table(&[("priority", "9")], &[])).document();
    let error = &doc["error"];
    assert_eq!(error["kind"], "usage");
    assert_eq!(error["usage"]["problem"], "bad_value");
    assert_eq!(error["usage"]["word"], "--priority");
    assert_eq!(error["selection"]["field"], "priority");
    assert_eq!(error["selection"]["value"], "9");
    assert_eq!(error["selection"]["command"], "dam list --priority 1");
    assert!(error["selection"]["shapes"].as_array().unwrap().len() > 2);
}

#[test]
fn a_mistyped_field_in_any_names_the_closest_real_one() {
    let text = read(table(&[], &[("lables", "x"), ("due", "today")])).to_string();
    assert!(
        text.starts_with("--any lables=x names no field; the closest is labels"),
        "{text}"
    );
    assert!(
        text.ends_with("; run dam list --any labels=x --any due=today"),
        "{text}"
    );
}

#[test]
fn one_field_asked_to_be_two_things_is_shown_the_list_or_any() {
    let list = read(table(&[("priority", "1"), ("priority", "2")], &[])).to_string();
    assert!(list.ends_with("; run dam list --priority 1,2"), "{list}");
    let dates = read(table(&[("due", "today"), ("due", "today+1")], &[])).to_string();
    assert!(
        dates.ends_with("; run dam list --any due=today --any due=today+1"),
        "{dates}"
    );
}

#[test]
fn a_kind_that_contradicts_a_field_is_dropped_from_the_command() {
    let typed = table(&[("kind", "task"), ("start", "today")], &[]);
    let e = defaults(DefaultsError::KindContradicts { field: "start" }, typed);
    assert_eq!(e.exit_code(), 2);
    let text = e.to_string();
    assert!(text.starts_with("--start lists only events"), "{text}");
    assert!(text.ends_with("; run dam list --start today"), "{text}");
    assert_eq!(e.document()["error"]["usage"]["problem"], "conflict");
}

#[test]
fn a_task_field_beside_an_event_field_is_pointed_at_any() {
    let typed = table(&[("due", "today"), ("start", "week")], &[]);
    let both = DefaultsError::BothKinds {
        task_field: "due",
        event_field: "start",
    };
    let text = defaults(both, typed).to_string();
    assert!(
        text.ends_with("; run dam list --any due=today --any start=week"),
        "{text}"
    );
}

#[test]
fn a_value_with_a_space_is_quoted_in_the_command() {
    let text = read(table(&[("path", "kitchen remodel/"), ("due", "x")], &[])).to_string();
    assert!(text.ends_with("--path 'kitchen remodel/'"), "{text}");
}
