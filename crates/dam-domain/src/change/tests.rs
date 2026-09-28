use super::*;
use crate::{Event, Field, Object, Oid, Priority, Task, When};

fn oid(b: u8) -> Oid {
    Oid::generate(&mut |x: &mut [u8]| x.fill(b))
}

fn task(subject: &str) -> Object {
    Object::Task(Task::new(oid(1), subject))
}

#[test]
fn diff_reports_create_update_delete_and_nothing() {
    let a = task("a");
    let b = task("b");
    assert_eq!(diff(&oid(1), None, Some(&a)).unwrap().op, Op::Create);
    assert_eq!(diff(&oid(1), Some(&a), Some(&b)).unwrap().op, Op::Update);
    assert_eq!(diff(&oid(1), Some(&a), None).unwrap().op, Op::Delete);
    assert!(diff(&oid(1), Some(&a), Some(&a)).is_none());
    assert!(diff(&oid(1), None, None).is_none());
}

#[test]
fn coalesce_create_then_update_is_a_create_of_the_final_state() {
    let a = task("a");
    let b = task("b");
    let c = diff(&oid(1), None, Some(&a)).unwrap();
    let u = diff(&oid(1), Some(&a), Some(&b)).unwrap();
    let out = coalesce(c, u).unwrap();
    assert_eq!(out.op, Op::Create);
    assert_eq!(out.after, Some(b));
}

#[test]
fn coalesce_create_then_delete_is_nothing() {
    let a = task("a");
    let c = diff(&oid(1), None, Some(&a)).unwrap();
    let d = diff(&oid(1), Some(&a), None).unwrap();
    assert!(coalesce(c, d).is_none());
}

#[test]
fn coalesce_update_then_update_keeps_the_first_before() {
    let a = task("a");
    let b = task("b");
    let c = task("c");
    let u1 = diff(&oid(1), Some(&a), Some(&b)).unwrap();
    let u2 = diff(&oid(1), Some(&b), Some(&c)).unwrap();
    let out = coalesce(u1, u2).unwrap();
    assert_eq!(out.op, Op::Update);
    assert_eq!(out.before, Some(a));
    assert_eq!(out.after, Some(c));
}

#[test]
fn coalesce_update_then_delete_is_a_delete_of_the_original() {
    let a = task("a");
    let b = task("b");
    let u = diff(&oid(1), Some(&a), Some(&b)).unwrap();
    let d = diff(&oid(1), Some(&b), None).unwrap();
    let out = coalesce(u, d).unwrap();
    assert_eq!(out.op, Op::Delete);
    assert_eq!(out.before, Some(a));
}

#[test]
fn changed_fields_names_what_moved() {
    let a = task("a");
    let mut t = Task::new(oid(1), "a");
    t.priority = Priority::HIGHEST;
    t.done = true;
    let b = Object::Task(t);
    assert_eq!(changed_fields(&a, &b), vec![Field::Priority, Field::Done]);
    assert!(changed_fields(&a, &a).is_empty());
}

#[test]
fn changed_fields_reports_kind_when_a_task_and_an_event_are_compared() {
    let start = When::Day(jiff::civil::date(2026, 9, 25));
    let event = Object::Event(Event::new(oid(1), "a", start.clone(), start));
    assert_eq!(changed_fields(&task("a"), &event), vec![Field::Kind]);
}
