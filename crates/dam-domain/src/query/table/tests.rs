use crate::select::{FilterTable, TableValue};
use crate::{Categories, query_table};

fn written(text: &str) -> Option<Vec<(String, String)>> {
    let table = query_table(text, &Categories::default())?;
    assert!(table.any.is_empty());
    Some(pairs(&table))
}

fn pairs(table: &FilterTable) -> Vec<(String, String)> {
    table
        .fields
        .iter()
        .map(|(k, v)| match v {
            TableValue::One(text) => (k.clone(), text.clone()),
            TableValue::Many(texts) => (k.clone(), texts.join(" & ")),
        })
        .collect()
}

fn owned(pairs: &[(&str, &str)]) -> Option<Vec<(String, String)>> {
    Some(
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    )
}

#[test]
fn a_task_field_needs_no_kind_and_done_tasks_stay_listed() {
    assert_eq!(
        written("due:today"),
        owned(&[("done", "any"), ("due", "today")])
    );
    assert_eq!(
        written("!done"),
        owned(&[("done", "false"), ("kind", "any")])
    );
}

#[test]
fn done_alone_means_done_tasks() {
    assert_eq!(
        written("done"),
        owned(&[("done", "true"), ("kind", "task")])
    );
}

#[test]
fn a_label_twice_repeats_the_field_and_odd_text_is_kept_exact() {
    assert_eq!(
        written("@a & @\"b,c\""),
        owned(&[("done", "any"), ("kind", "any"), ("labels", "a & =b,c")])
    );
}

#[test]
fn what_the_table_cannot_say_has_no_translation() {
    for text in [
        "!overdue",
        "!due:none",
        "(@a | @b) & (@c | @d)",
        "due:soon",
        "",
    ] {
        assert_eq!(written(text), None, "{text}");
    }
}

#[test]
fn an_or_becomes_any() {
    let table = query_table("@a | due:today", &Categories::default()).unwrap();
    let alternatives: Vec<_> = table.any.iter().map(pairs).collect();
    assert_eq!(
        alternatives,
        [
            owned(&[("labels", "a")]).unwrap(),
            owned(&[("due", "today")]).unwrap()
        ]
    );
}
