use super::*;
use crate::{Field, Object, Oid, Path, Priority, Task, When};
use jiff::civil::date;

fn oid(b: u8) -> Oid {
    Oid::generate(&mut |x: &mut [u8]| x.fill(b))
}

fn task(subject: &str) -> Object {
    Object::Task(Task::new(oid(1), subject))
}

#[test]
fn an_edited_field_is_the_only_one_named() {
    let before = task("a");
    let after = task("b");
    let change = diff(&oid(1), Some(&before), Some(&after)).unwrap();
    assert_eq!(touched_fields(&change), vec![Field::Subject]);
}

#[test]
fn reopening_a_task_names_done_alone() {
    let mut done = Task::new(oid(1), "a");
    done.done = true;
    let before = Object::Task(done);
    let after = task("a");
    let change = diff(&oid(1), Some(&before), Some(&after)).unwrap();
    assert_eq!(touched_fields(&change), vec![Field::Done]);
}

#[test]
fn a_create_names_every_field_it_sets_and_no_default() {
    let mut t = Task::new(oid(1), "buy oat milk");
    t.priority = Priority::new(1).unwrap();
    t.due = Some(When::Day(date(2026, 9, 25)));
    t.base.path = Path::parse("work").unwrap();
    let change = diff(&oid(1), None, Some(&Object::Task(t))).unwrap();
    assert_eq!(
        touched_fields(&change),
        vec![Field::Subject, Field::Path, Field::Priority, Field::Due]
    );
}

#[test]
fn a_bare_create_names_its_subject_alone() {
    let change = diff(&oid(1), None, Some(&task("a"))).unwrap();
    assert_eq!(touched_fields(&change), vec![Field::Subject]);
}

#[test]
fn a_delete_names_nothing() {
    let change = diff(&oid(1), Some(&task("a")), None).unwrap();
    assert!(touched_fields(&change).is_empty());
}
