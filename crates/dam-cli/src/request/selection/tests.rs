use std::ffi::OsString;

use dam_domain::{Filter, FilterTable, TableValue};
use serde_json::{Value, json};
use tempfile::NamedTempFile;

use crate::args::{Command, ListArgs};
use crate::request::parse;
use crate::testing::list_args;

fn requested(words: &[&str], request: &str) -> Result<ListArgs, String> {
    let file = NamedTempFile::new().unwrap();
    std::fs::write(file.path(), request).unwrap();
    let line = ["dam", "list"]
        .iter()
        .chain(words)
        .map(OsString::from)
        .chain([OsString::from("--request"), file.path().into()])
        .collect();
    match parse(line)
        .map_err(|refused| refused.error.to_string())?
        .command
    {
        Command::List(args) => Ok(args),
        other => panic!("not list: {other:?}"),
    }
}

fn by_json(request: Value) -> Result<ListArgs, String> {
    requested(&[], &request.to_string())
}

fn one(text: &str) -> TableValue {
    TableValue::One(text.into())
}

fn table(pairs: &[(&str, TableValue)], any: Vec<FilterTable>) -> FilterTable {
    FilterTable {
        fields: pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect(),
        any,
    }
}

#[test]
fn a_request_reads_into_the_same_selection_and_order_the_flags_give() {
    let request = json!({
        "kind": "task",
        "done": true,
        "subject": "report",
        "path": "work/",
        "labels": ["bug", "not:waiting"],
        "priority": "1,2",
        "estimate": "..2h",
        "completed_at": "week",
        "author": "owner",
        "overdue": false,
        "blocked": "any",
        "deadline": "none",
        "any": [{"labels": "urgent"}, {"due": "..today"}],
        "sort": ["priority", "due:desc"],
        "limit": 10,
        "offset": 2
    });
    let flags = list_args(&[
        "--kind",
        "task",
        "--done",
        "--subject",
        "report",
        "--path",
        "work/",
        "--labels",
        "bug",
        "--labels",
        "not:waiting",
        "--priority",
        "1,2",
        "--estimate",
        "..2h",
        "--completed-at",
        "week",
        "--author",
        "owner",
        "--overdue",
        "false",
        "--blocked",
        "any",
        "--deadline",
        "none",
        "--any",
        "labels=urgent",
        "--any",
        "due=..today",
        "--sort",
        "priority",
        "--sort",
        "due:desc",
        "--limit",
        "10",
        "--offset",
        "2",
    ]);
    let read = by_json(request).unwrap();
    assert_eq!(read.selection.typed, flags.selection.typed);
    assert_eq!(read.order.sort, flags.order.sort);
    assert_eq!((read.order.limit, read.order.offset), (Some(10), Some(2)));
}

#[test]
fn any_holds_tables_of_several_fields_that_nest_and_survive_a_round_trip() {
    let request = json!({
        "path": "work/",
        "any": [
            {"labels": "urgent", "priority": 1},
            {"any": [{"due": "..today"}, {"kind": "event", "start": "today"}]}
        ]
    });
    let expected = table(
        &[("path", one("work/"))],
        vec![
            table(&[("labels", one("urgent")), ("priority", one("1"))], vec![]),
            table(
                &[],
                vec![
                    table(&[("due", one("..today"))], vec![]),
                    table(&[("kind", one("event")), ("start", one("today"))], vec![]),
                ],
            ),
        ],
    );
    let typed = by_json(request).unwrap().selection.typed;
    assert_eq!(typed, expected);
    assert_eq!(Filter::read(&typed).unwrap().write(), typed);
}

#[test]
fn toon_reads_like_json() {
    let toon = "due: ..today\npriority: 1\n";
    let typed = requested(&[], toon).unwrap().selection.typed;
    assert_eq!(
        typed,
        list_args(&["--due", "..today", "--priority", "1"])
            .selection
            .typed
    );
}

#[test]
fn an_unknown_key_names_the_closest_and_nothing_else_is_guessed() {
    let err = by_json(json!({"duee": "today"})).unwrap_err();
    assert_eq!(
        err,
        "--request: duee is not a key dam list takes; the closest is due"
    );
    let nested = by_json(json!({"any": [{"sort": "due"}]})).unwrap_err();
    assert!(
        nested.starts_with("--request: sort is not a key"),
        "{nested}"
    );
}

#[test]
fn a_list_of_numbers_on_a_one_value_field_is_read_as_two_at_once() {
    let typed = by_json(json!({"priority": [1, 2]}))
        .unwrap()
        .selection
        .typed;
    let two = Filter::read(&typed).unwrap_err();
    assert_eq!(two.field, "priority");
}

#[test]
fn shapes_a_request_cannot_take_are_refused_with_the_shape_it_takes() {
    let limit = by_json(json!({"limit": -1})).unwrap_err();
    assert_eq!(limit, "--request: limit must be a whole number");
    let any = by_json(json!({"any": {"labels": "x"}})).unwrap_err();
    assert!(
        any.starts_with("--request: any must be a list of tables"),
        "{any}"
    );
    let nested = by_json(json!({"labels": {"a": "b"}})).unwrap_err();
    assert!(
        nested.starts_with("--request: labels takes text"),
        "{nested}"
    );
}

#[test]
fn scope_keys_go_through_and_selection_flags_beside_a_request_are_refused() {
    assert!(by_json(json!({"global": true})).unwrap().scope.global);
    let beside = requested(&["--due", "today"], "{}").unwrap_err();
    assert_eq!(
        beside,
        "--request takes only global flags beside it; put due in the request"
    );
}
