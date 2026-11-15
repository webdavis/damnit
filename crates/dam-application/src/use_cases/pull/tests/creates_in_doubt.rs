use dam_domain::Object;

use super::unanswered::{pull, pulled, push_accepting, push_with_no_answer, subject};
use crate::ports::AttemptRepository;
use crate::remote::MutationOp;
use crate::testing::prelude::*;
use crate::testing::{Scenario, oid, task_in, todoist};

fn ada(byte: u8, subject: &str) -> Object {
    Object::Task(task_in(byte, subject, "work/"))
}

fn created_with_no_answer() -> Scenario {
    let s = Scenario::default().drawing_from(42);
    s.committed(ada(1, "call Ada"));
    push_with_no_answer(&s);
    s
}

fn mapped_to(s: &Scenario, remote_id: &str) -> Option<dam_domain::Oid> {
    s.store.oid_for_remote_id(&todoist(), remote_id).unwrap()
}

#[test]
fn a_pulled_create_in_doubt_is_adopted_and_not_sent_again() {
    let s = created_with_no_answer();

    pull(&s, vec![pulled(ada(0, "call Ada"), "i:9")]);

    assert_eq!(s.store.all().unwrap(), vec![ada(1, "call Ada")]);
    assert_eq!(mapped_to(&s, "i:9"), Some(oid(1)));
    assert_eq!(push_accepting(&s, "i:9"), vec![]);
    assert!(s.store.attempts(&todoist()).unwrap().is_empty());
}

#[test]
fn an_edit_here_after_the_create_is_kept_and_sent_as_an_update() {
    let s = created_with_no_answer();
    s.committed(ada(1, "call Ada at 5"));

    pull(&s, vec![pulled(ada(0, "call Ada"), "i:9")]);

    assert_eq!(subject(&s), "call Ada at 5");
    let sent = push_accepting(&s, "i:9");
    let update = (
        MutationOp::Update,
        Some("i:9".into()),
        Some("call Ada at 5".into()),
    );
    assert_eq!(sent, vec![update]);
    assert_eq!(s.store.all().unwrap().len(), 1);
}

#[test]
fn one_pulled_task_for_two_equal_creates_is_not_adopted() {
    let s = Scenario::default().drawing_from(42);
    s.committed(ada(1, "call Ada"));
    s.committed(ada(2, "call Ada"));
    push_with_no_answer(&s);

    pull(&s, vec![pulled(ada(0, "call Ada"), "i:9")]);

    assert_eq!(mapped_to(&s, "i:9"), Some(oid(42)));
}

#[test]
fn a_task_edited_in_the_app_before_the_pull_is_not_adopted() {
    let s = created_with_no_answer();

    pull(&s, vec![pulled(ada(0, "call Ada today"), "i:9")]);

    assert_eq!(mapped_to(&s, "i:9"), Some(oid(42)));
    assert_eq!(subject(&s), "call Ada");
}

#[test]
fn a_create_removed_here_is_not_adopted() {
    let s = created_with_no_answer();
    s.deleted(&oid(1));

    pull(&s, vec![pulled(ada(0, "call Ada"), "i:9")]);

    assert_eq!(mapped_to(&s, "i:9"), Some(oid(42)));
    assert!(s.store.get(&oid(1)).unwrap().is_none());
}

#[test]
fn a_create_removed_here_and_not_committed_is_not_adopted() {
    let s = created_with_no_answer();
    s.store.delete(&oid(1)).unwrap();

    pull(&s, vec![pulled(ada(0, "call Ada"), "i:9")]);

    assert_eq!(mapped_to(&s, "i:9"), Some(oid(42)));
    assert!(s.store.get(&oid(1)).unwrap().is_none());
}
