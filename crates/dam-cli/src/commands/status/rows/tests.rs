use dam_domain::{Event, Object, Oid, Path, Priority, Task, When, diff};
use jiff::civil::date;

use super::change_json;

fn oid(byte: u8) -> Oid {
    Oid::generate(&mut |x: &mut [u8]| x.fill(byte))
}

fn task(subject: &str) -> Object {
    Object::Task(Task::new(oid(1), subject))
}

/// A task carrying something in every field a change row reports.
fn furnished_task() -> Object {
    let mut t = Task::new(oid(1), "buy oat milk");
    t.priority = Priority::new(1).unwrap();
    t.due = Some(When::Day(date(2026, 9, 25)));
    t.base.path = Path::parse("work").unwrap();
    t.base.labels.insert("errand".into());
    Object::Task(t)
}

#[test]
fn an_edit_names_the_field_it_moved() {
    let change = diff(&oid(1), Some(&task("a")), Some(&task("b"))).unwrap();
    let document = change_json(&change, false).unwrap();
    assert_eq!(document["fields"], serde_json::json!(["subject"]));
}

#[test]
fn a_reopened_task_names_done() {
    let mut completed = Task::new(oid(1), "a");
    completed.done = true;
    let change = diff(&oid(1), Some(&Object::Task(completed)), Some(&task("a"))).unwrap();
    assert_eq!(
        change_json(&change, false).unwrap()["fields"],
        serde_json::json!(["done"])
    );
}

#[test]
fn a_create_names_every_field_it_sets() {
    let change = diff(&oid(1), None, Some(&furnished_task())).unwrap();
    assert_eq!(
        change_json(&change, false).unwrap()["fields"],
        serde_json::json!(["subject", "path", "labels", "priority", "due"])
    );
}

#[test]
fn a_delete_names_no_field() {
    let change = diff(&oid(1), Some(&task("a")), None).unwrap();
    assert_eq!(
        change_json(&change, false).unwrap()["fields"],
        serde_json::json!([])
    );
}

#[test]
fn the_default_row_is_the_after_state_without_the_object() {
    let change = diff(&oid(1), None, Some(&furnished_task())).unwrap();
    assert_eq!(
        change_json(&change, false).unwrap(),
        serde_json::json!({
            "oid": oid(1).to_string(),
            "op": "create",
            "fields": ["subject", "path", "labels", "priority", "due"],
            "kind": "task",
            "subject": "buy oat milk",
            "path": "work/",
            "labels": ["errand"],
            "done": false,
            "completed_at": null,
            "priority": 1,
            "due": "2026-09-25",
        })
    );
}

#[test]
fn the_full_row_is_the_same_row_plus_the_objects() {
    let after = furnished_task();
    let change = diff(&oid(1), None, Some(&after)).unwrap();
    let full = change_json(&change, true).unwrap();
    let mut expected = change_json(&change, false).unwrap();
    expected["before"] = serde_json::Value::Null;
    expected["after"] = crate::output::object_json(&after).unwrap();
    assert_eq!(full, expected);
    assert_eq!(full["after"]["body"], "");
}

#[test]
fn an_event_row_carries_its_start_and_end_in_place_of_the_task_fields() {
    let event = Object::Event(Event::new(
        oid(2),
        "dentist",
        When::Day(date(2026, 9, 25)),
        When::Day(date(2026, 9, 26)),
    ));
    let change = diff(&oid(2), None, Some(&event)).unwrap();
    assert_eq!(
        change_json(&change, false).unwrap(),
        serde_json::json!({
            "oid": oid(2).to_string(),
            "op": "create",
            "fields": ["subject", "start", "end"],
            "kind": "event",
            "subject": "dentist",
            "path": "",
            "labels": [],
            "start": "2026-09-25",
            "end": "2026-09-26",
        })
    );
}

#[test]
fn a_delete_describes_the_object_it_removed() {
    let change = diff(&oid(1), Some(&furnished_task()), None).unwrap();
    let document = change_json(&change, false).unwrap();
    assert_eq!(document["op"], "delete");
    assert_eq!(document["subject"], "buy oat milk");
    assert_eq!(document["fields"], serde_json::json!([]));
}

/// A change row carries the completion time beside `done`, so a Done list
/// renders from `status` without a second read of the log.
#[test]
fn a_completed_task_row_carries_when_it_was_completed() {
    let mut completed = Task::new(oid(1), "a");
    completed.done = true;
    completed.completed_at = Some("2026-09-18T15:04:05Z".parse().unwrap());
    let change = diff(&oid(1), Some(&task("a")), Some(&Object::Task(completed))).unwrap();
    let document = change_json(&change, false).unwrap();
    assert_eq!(document["done"], true);
    assert_eq!(document["completed_at"], "2026-09-18T15:04:05Z");
}

#[test]
fn an_open_task_row_carries_a_null_completion_time() {
    let change = diff(&oid(1), None, Some(&task("a"))).unwrap();
    let document = change_json(&change, false).unwrap();
    assert_eq!(document["completed_at"], serde_json::Value::Null);
}
