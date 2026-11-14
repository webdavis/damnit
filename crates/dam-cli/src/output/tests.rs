use super::*;
use crate::view::{ObjectRow, report};
use dam_domain::{Oid, Path, Priority, Task, When};
use jiff::civil::date;

#[test]
fn an_object_row_has_short_oid_priority_due_path_then_subject_and_labels() {
    let mut t = Task::new(
        Oid::generate(&mut |x: &mut [u8]| x.fill(0x3f)),
        "buy oat milk",
    );
    t.priority = Priority::new(2).unwrap();
    t.due = Some(When::Day(date(2026, 9, 25)));
    t.base.labels.insert("errand".into());
    t.base.path = Path::parse("work").unwrap();
    let line = report(&ObjectRow(&Object::Task(t), &TimeZone::UTC))
        .unwrap()
        .human;
    assert_eq!(
        line,
        "3f3f3f3  p2  2026-09-25  work/  buy oat milk  [errand]"
    );
}

#[test]
fn a_timed_event_reads_in_local_time() {
    let at = "2026-10-13T14:00:00+00:00[UTC]".parse().unwrap();
    let zone = TimeZone::fixed(jiff::tz::offset(-4));
    assert_eq!(local_text(&When::At(at), &zone), "2026-10-13 10:00");
}

#[test]
fn toon_renders_a_list_as_a_table() {
    let data = serde_json::json!({"objects": [{"oid": "a", "subject": "x"}, {"oid": "b", "subject": "y"}]});
    let text = render(
        Format::Toon,
        &Report {
            human: String::new(),
            data,
            cut: Vec::new(),
            tints: Vec::new(),
            standard_error_lines: Vec::new(),
        },
    )
    .unwrap();
    assert!(text.starts_with("objects[2]{oid,subject}:"), "{text}");
}

#[test]
fn json_renders_the_data_and_human_renders_the_text() {
    let report = Report {
        human: "hi".into(),
        data: serde_json::json!({"a": 1}),
        cut: Vec::new(),
        tints: Vec::new(),
        standard_error_lines: Vec::new(),
    };
    assert_eq!(render(Format::Human, &report).unwrap(), "hi");
    assert_eq!(render(Format::Json, &report).unwrap(), "{\n  \"a\": 1\n}");
}
