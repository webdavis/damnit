use dam_domain::Permission::{TaskChange, TaskRead};

use super::*;
use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;
use crate::testing::prelude::ObjectRepository;
use crate::testing::{allowed, holder};
use crate::use_cases::each::SetRun;

fn labels_of(s: &Scenario, byte: u8) -> Vec<String> {
    s.store
        .get(&oid(byte))
        .unwrap()
        .unwrap()
        .base()
        .labels
        .iter()
        .cloned()
        .collect()
}

#[test]
fn every_object_carrying_an_old_label_carries_the_new_one_instead() {
    let s = scenario(vec![
        labelled(1, &["quick", "home"]),
        labelled(2, &["hard"]),
        labelled(3, &["home"]),
    ]);
    let out = s
        .dam()
        .migrate_labels(
            &map(&[("quick", "light"), ("hard", "deep")]),
            SetRun::default(),
        )
        .unwrap();
    assert_eq!(out.oids, vec![oid(1), oid(2)]);
    assert_eq!(out.changes.len(), 2);
    assert_eq!(labels_of(&s, 1), vec!["home", "light"]);
    assert_eq!(labels_of(&s, 2), vec!["deep"]);
    assert_eq!(labels_of(&s, 3), vec!["home"]);
}

#[test]
fn a_preview_answers_the_changes_and_writes_nothing() {
    let s = scenario(vec![labelled(1, &["quick"])]);
    let before = s.store.clone();
    let run = SetRun {
        preview: true,
        as_one: false,
        expect: None,
    };
    let out = s
        .dam()
        .migrate_labels(&map(&[("quick", "light")]), run)
        .unwrap();
    assert!(out.preview);
    assert_eq!(out.changes.len(), 1);
    assert_eq!(s.store, before);
}

#[test]
fn an_expected_count_that_differs_refuses_and_changes_nothing() {
    let s = scenario(vec![labelled(1, &["quick"]), labelled(2, &["quick"])]);
    let run = SetRun {
        preview: false,
        as_one: false,
        expect: Some(1),
    };
    assert_eq!(
        s.dam().migrate_labels(&map(&[("quick", "light")]), run),
        Err(UseCaseError::Refused(Refusal::CountMismatch {
            expected: 1,
            count: 2
        }))
    );
    assert_eq!(labels_of(&s, 1), vec!["quick"]);
}

#[test]
fn a_result_holding_two_labels_of_an_exclusive_category_refuses_naming_every_object() {
    let s = scenario(vec![
        labelled(1, &["urgent", "hard"]),
        labelled(2, &["urgent"]),
        labelled(3, &["urgent", "deep"]),
    ]);
    assert_eq!(
        s.dam()
            .migrate_labels(&map(&[("urgent", "quick")]), SetRun::default()),
        Err(UseCaseError::Refused(Refusal::ExclusiveAfterMigrate {
            category: "effort".into(),
            oids: vec![oid(1), oid(3)],
        }))
    );
    assert_eq!(labels_of(&s, 2), vec!["urgent"]);
}

#[test]
fn a_token_is_refused_before_any_object_is_looked_at() {
    let s = scenario(vec![labelled(1, &["quick"])]);
    let token = holder(vec![allowed(&[TaskRead, TaskChange], None)]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    assert_eq!(
        dam.migrate_labels(&map(&[("quick", "light")]), SetRun::default()),
        Err(UseCaseError::Refused(Refusal::NotOpenToTokens(
            "category migrate"
        )))
    );
}
