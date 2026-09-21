use dam_domain::{Object, Oid, Path, Priority, Task, When, diff};
use jiff::civil::date;

use super::change_json;

fn oid(byte: u8) -> Oid {
    Oid::generate(&mut |x: &mut [u8]| x.fill(byte))
}

fn task(subject: &str) -> Object {
    Object::Task(Task::new(oid(1), subject))
}

#[test]
fn an_edit_names_the_field_it_moved() {
    let change = diff(&oid(1), Some(&task("a")), Some(&task("b"))).unwrap();
    let document = change_json(&change).unwrap();
    assert_eq!(document["fields"], serde_json::json!(["subject"]));
}

#[test]
fn a_reopened_task_names_done() {
    let mut completed = Task::new(oid(1), "a");
    completed.done = true;
    let change = diff(&oid(1), Some(&Object::Task(completed)), Some(&task("a"))).unwrap();
    assert_eq!(
        change_json(&change).unwrap()["fields"],
        serde_json::json!(["done"])
    );
}

#[test]
fn a_create_names_every_field_it_sets() {
    let mut t = Task::new(oid(1), "buy oat milk");
    t.priority = Priority::new(1).unwrap();
    t.due = Some(When::Day(date(2026, 9, 25)));
    t.base.path = Path::parse("work").unwrap();
    let change = diff(&oid(1), None, Some(&Object::Task(t))).unwrap();
    assert_eq!(
        change_json(&change).unwrap()["fields"],
        serde_json::json!(["subject", "path", "priority", "due"])
    );
}

#[test]
fn a_delete_names_no_field() {
    let change = diff(&oid(1), Some(&task("a")), None).unwrap();
    assert_eq!(
        change_json(&change).unwrap()["fields"],
        serde_json::json!([])
    );
}
