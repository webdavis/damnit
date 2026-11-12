use super::proposal::same_proposal;
use crate::{Change, Completion, Object, Oid, Task, Timestamp};

fn task(byte: u8, subject: &str) -> Task {
    Task::new(Oid::generate(&mut |b| b.fill(byte)), subject)
}

fn create(byte: u8, subject: &str) -> Change {
    Change::Create {
        after: Object::Task(task(byte, subject)),
    }
}

fn completed(at: &str) -> Change {
    let before = task(1, "pay rent");
    let mut after = before.clone();
    after.completion = Completion::Done {
        at: Some(at.parse::<Timestamp>().unwrap()),
    };
    Change::Update {
        before: Object::Task(before),
        after: Object::Task(after),
    }
}

#[test]
fn two_new_tasks_alike_but_for_their_oids_are_the_same_proposal() {
    assert!(same_proposal(&create(1, "milk"), &create(2, "milk")));
    assert!(!same_proposal(&create(1, "milk"), &create(2, "bread")));
}

#[test]
fn completing_one_task_twice_is_the_same_proposal_whenever_it_was_sent() {
    assert!(same_proposal(
        &completed("2026-11-11T09:00:00Z"),
        &completed("2026-11-11T10:00:00Z")
    ));
    assert!(!same_proposal(
        &completed("2026-11-11T09:00:00Z"),
        &create(1, "pay rent")
    ));
}

#[test]
fn a_change_to_another_field_or_task_is_another_proposal() {
    let before = task(1, "pay rent");
    let mut retitled = before.clone();
    retitled.base.subject = "pay the rent".into();
    let mut moved = before.clone();
    moved.base.body = "by friday".into();
    let change = |after: &Task| Change::Update {
        before: Object::Task(before.clone()),
        after: Object::Task(after.clone()),
    };
    assert!(same_proposal(&change(&retitled), &change(&retitled)));
    assert!(!same_proposal(&change(&retitled), &change(&moved)));
}
