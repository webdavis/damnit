use std::collections::BTreeSet;

use dam_domain::Permission::{TaskChange, TaskComplete};
use dam_domain::{Change, Force, Grant, Object, Tracked};

use super::{Answer, Landed, Sent};
use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;
use crate::testing::prelude::*;
use crate::testing::{Scenario, allowed, holder, oid, task_in};
use crate::use_cases::edit::EditFields;
use crate::use_cases::items::refusal::ItemRefusal;

fn sent(landed: Landed) -> Sent {
    match landed {
        Landed::Sent(sent) => sent,
        Landed::Written(object) => panic!("a token's change was written: {object:?}"),
    }
}

fn retitled(subject: &str) -> EditFields {
    EditFields {
        subject: Some(subject.into()),
        ..EditFields::default()
    }
}

fn pay_rent() -> Scenario {
    let s = Scenario::default().drawing_from(7);
    s.committed(Object::Task(task_in(1, "pay rent", "home")));
    s
}

#[test]
fn a_tokens_edit_waits_as_a_change_item_and_leaves_the_task_as_it_was() {
    let s = pay_rent();
    let vpt = holder(vec![allowed(&[TaskChange], None)]);
    let dam = Dam {
        access: &vpt,
        ..s.dam()
    };
    let sent = sent(dam.edit(&oid(1), retitled("pay the rent")).unwrap());
    assert_eq!(sent.answer, Answer::Changed);
    assert_eq!(sent.item.target(), &oid(1));
    let Change::Update { before, after } = &sent.item.change else {
        panic!("not a change: {:?}", sent.item.change)
    };
    assert_eq!(before.base().subject, "pay rent");
    assert_eq!(after.base().subject, "pay the rent");
    assert_eq!(
        s.store.get(&oid(1)).unwrap().unwrap().base().subject,
        "pay rent"
    );
}

#[test]
fn a_tokens_done_waits_as_a_change_item_and_the_task_stays_open() {
    let s = pay_rent();
    let vpt = holder(vec![allowed(&[TaskComplete], None)]);
    let dam = Dam {
        access: &vpt,
        ..s.dam()
    };
    let (_, landed) = dam.complete(&oid(1), Force::No, Tracked::No).unwrap();
    let sent = sent(landed);
    assert!(
        sent.item
            .change
            .after()
            .unwrap()
            .as_task()
            .unwrap()
            .completion
            .is_done()
    );
    assert!(
        !s.store
            .get(&oid(1))
            .unwrap()
            .unwrap()
            .as_task()
            .unwrap()
            .completion
            .is_done()
    );
    assert_eq!(
        dam.complete(&oid(1), Force::With(Default::default()), Tracked::No),
        Err(Refusal::NotOpenToTokens("done --force").into())
    );
}

#[test]
fn a_change_that_leaves_the_grant_after_it_is_refused() {
    let s = Scenario::default().drawing_from(7);
    let mut errand = task_in(1, "buy milk", "home");
    errand.base.labels = BTreeSet::from(["vpt".to_string()]);
    s.committed(Object::Task(errand));
    let labelled = holder(vec![Grant {
        labels: Some(BTreeSet::from(["vpt".to_string()])),
        ..allowed(&[TaskChange], None)
    }]);
    let dam = Dam {
        access: &labelled,
        ..s.dam()
    };
    let unlabel = EditFields {
        remove_labels: vec!["vpt".into()],
        ..EditFields::default()
    };
    assert_eq!(
        dam.edit(&oid(1), unlabel),
        Err(UseCaseError::Refused(Refusal::NotPermitted(TaskChange)))
    );
    assert!(s.store.items().unwrap().is_empty());
}

#[test]
fn an_edit_that_changes_nothing_is_refused() {
    let s = pay_rent();
    let vpt = holder(vec![allowed(&[TaskChange], None)]);
    let dam = Dam {
        access: &vpt,
        ..s.dam()
    };
    assert_eq!(
        dam.edit(&oid(1), retitled("pay rent")),
        Err(ItemRefusal::NothingChanged.into())
    );
}
