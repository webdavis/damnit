use std::collections::BTreeSet;

use dam_domain::Permission::{TaskCreate, TaskRead};
use dam_domain::{Outcome, Path, Priority};

use super::{Answer, Landed, Sent};
use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;
use crate::testing::prelude::*;
use crate::testing::{Scenario, allowed, holder};
use crate::use_cases::items::refusal::ItemRefusal;
use crate::use_cases::new_object::NewTask;

fn new_task(subject: &str, path: &str) -> NewTask {
    NewTask {
        subject: subject.into(),
        path: Path::parse(path).unwrap(),
        priority: Priority::default(),
        due: None,
        deadline: None,
        labels: BTreeSet::new(),
        body: String::new(),
        id: None,
    }
}

fn sent(landed: Result<Landed, UseCaseError>) -> Sent {
    match landed.unwrap() {
        Landed::Sent(sent) => sent,
        Landed::Written(object) => panic!("a token's task was written: {object:?}"),
    }
}

#[test]
fn a_tokens_new_task_waits_as_an_item_and_touches_nothing_of_the_owners() {
    let s = Scenario::default().drawing_from(7);
    let tool = holder(vec![allowed(&[TaskCreate], None)]);
    let dam = Dam {
        access: &tool,
        ..s.dam()
    };
    let sent = sent(dam.new_task(new_task("call the dentist", "inbox")));
    assert_eq!(sent.answer, Answer::New);
    assert_eq!(sent.item.author.name, "tool");
    assert_eq!(sent.item.outcome, Outcome::Waiting);
    assert_eq!(sent.item.change.object().base().subject, "call the dentist");
    assert!(s.store.all().unwrap().is_empty());
    assert!(s.store.staged().unwrap().is_empty());
    assert_eq!(s.store.items().unwrap(), vec![sent.item]);
}

#[test]
fn the_same_task_sent_again_while_it_waits_is_a_duplicate_and_stores_nothing() {
    let s = Scenario::default().drawing_from(7);
    let tool = holder(vec![allowed(&[TaskCreate], None)]);
    let dam = Dam {
        access: &tool,
        ..s.dam()
    };
    let first = sent(dam.new_task(new_task("call the dentist", "inbox")));
    let again = sent(dam.new_task(new_task("call the dentist", "inbox")));
    assert_eq!(again.answer, Answer::Duplicate);
    assert_eq!(again.item.hash, first.item.hash);
    assert_eq!(s.store.items().unwrap().len(), 1);
}

#[test]
fn a_task_the_owner_declined_is_refused_when_sent_again() {
    let s = Scenario::default().drawing_from(7);
    let tool = holder(vec![allowed(&[TaskCreate], None)]);
    let dam = Dam {
        access: &tool,
        ..s.dam()
    };
    let first = sent(dam.new_task(new_task("call the dentist", "inbox")));
    let at = "2026-11-11T09:00:00Z".parse().unwrap();
    s.store
        .settle(&first.item.hash, Outcome::Declined(at))
        .unwrap();
    assert_eq!(
        dam.new_task(new_task("call the dentist", "inbox")),
        Err(ItemRefusal::Declined(first.item.hash).into())
    );
    assert_eq!(s.store.items().unwrap().len(), 1);
}

#[test]
fn a_task_outside_the_tokens_paths_or_permissions_is_refused_and_nothing_is_stored() {
    let s = Scenario::default().drawing_from(7);
    let work_only = holder(vec![allowed(&[TaskCreate], Some(&["work"]))]);
    let reader = holder(vec![allowed(&[TaskRead], None)]);
    for (token, path) in [(&work_only, "home"), (&reader, "work")] {
        let dam = Dam {
            access: token,
            ..s.dam()
        };
        assert_eq!(
            dam.new_task(new_task("laundry", path)),
            Err(UseCaseError::Refused(Refusal::NotPermitted(TaskCreate)))
        );
    }
    assert!(s.store.items().unwrap().is_empty());
}
