use super::*;
use crate::{Completion, Event, Field, Object, Oid, Priority, Task, When};

fn oid(b: u8) -> Oid {
    Oid::generate(&mut |x: &mut [u8]| x.fill(b))
}

fn task(subject: &str) -> Object {
    Object::Task(Task::new(oid(1), subject))
}

#[test]
fn a_create_then_an_update_nets_to_a_create_of_the_final_state() {
    assert_eq!(
        Change::between(None, Some(task("b"))),
        Some(Change::Create { after: task("b") })
    );
}

#[test]
fn a_create_then_a_delete_nets_to_nothing() {
    assert_eq!(Change::between(None, None), None);
}

#[test]
fn two_updates_net_to_one_from_the_first_before_to_the_last_after() {
    assert_eq!(
        Change::between(Some(task("a")), Some(task("c"))),
        Some(Change::Update {
            before: task("a"),
            after: task("c")
        })
    );
}

#[test]
fn an_update_then_a_delete_nets_to_a_delete_of_the_original() {
    assert_eq!(
        Change::between(Some(task("a")), None),
        Some(Change::Delete { before: task("a") })
    );
}

#[test]
fn an_edit_then_its_undo_nets_to_nothing() {
    assert_eq!(Change::between(Some(task("a")), Some(task("a"))), None);
}

#[test]
fn each_change_reads_its_op_sides_and_oid_from_its_variant() {
    let update = Change::Update {
        before: task("a"),
        after: task("b"),
    };
    assert_eq!(update.op(), Op::Update);
    assert_eq!(update.before(), Some(&task("a")));
    assert_eq!(update.after(), Some(&task("b")));
    assert_eq!(update.oid(), &oid(1));
    assert_eq!(update.object(), &task("b"));
    let delete = Change::Delete { before: task("a") };
    assert_eq!((delete.op(), delete.after()), (Op::Delete, None));
    assert_eq!(delete.object(), &task("a"));
    let create = Change::Create { after: task("a") };
    assert_eq!((create.op(), create.before()), (Op::Create, None));
}

#[test]
fn changed_fields_names_what_moved() {
    let a = task("a");
    let mut t = Task::new(oid(1), "a");
    t.priority = Priority::HIGHEST;
    t.completion = Completion::Done { at: None };
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

#[test]
fn a_change_to_the_completion_time_alone_names_done() {
    let mut before = Task::new(oid(1), "a");
    before.completion = Completion::Done { at: None };
    let mut after = before.clone();
    after.completion = Completion::Done {
        at: Some(crate::Timestamp::UNIX_EPOCH),
    };
    assert_eq!(
        changed_fields(&Object::Task(before), &Object::Task(after)),
        vec![Field::Done]
    );
}
