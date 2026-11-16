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
        "path:\"kitchen remodel/",
        "(due:today",
        "due:today |",
        "due:today due:tomorrow",
        "mood:happy",
        "p9",
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

#[test]
fn a_term_named_after_a_json_key_reads_as_its_short_form() {
    assert_eq!(written("labels:errand"), written("label:errand"));
    assert_eq!(written("completed_at:today"), written("completed:today"));
}

#[test]
fn a_quoted_value_keeps_spaces_and_escaped_characters() {
    assert_eq!(
        written(r#"path:"kitchen remodel/" & subject:"say \"hi\"""#),
        owned(&[
            ("done", "any"),
            ("kind", "any"),
            ("path", "kitchen remodel/"),
            ("subject", "say \"hi\""),
        ])
    );
}

#[test]
fn nesting_past_the_bound_has_no_translation_and_up_to_it_does() {
    let at_bound = format!("{}done{}", "(".repeat(64), ")".repeat(64));
    assert!(written(&at_bound).is_some());
    let past = format!("{}done{}", "(".repeat(65), ")".repeat(65));
    assert_eq!(written(&past), None);
    assert_eq!(written(&format!("{}done", "!".repeat(65))), None);
}
