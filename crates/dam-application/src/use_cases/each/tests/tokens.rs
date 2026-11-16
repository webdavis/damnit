use dam_domain::Permission::{TaskChange, TaskRead, TaskRemove};

use super::*;
use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;
use crate::testing::prelude::ItemRepository;
use crate::testing::prelude::ObjectRepository;
use crate::testing::{allowed, holder};

fn token_over(paths: &[&str]) -> crate::access::Holder {
    holder(vec![allowed(
        &[TaskRead, TaskChange, TaskRemove],
        Some(paths),
    )])
}

#[test]
fn a_token_that_sees_nothing_selects_nothing_and_learns_no_count() {
    let s = work_and_home();
    let token = token_over(&["garden"]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    let out = dam
        .edit_each(&picked(&[("kind", "task")]), retitled("x"), run())
        .unwrap();
    assert!(out.oids.is_empty());
    assert_eq!(
        dam.edit_each(&named(&[1]), retitled("x"), run()),
        Err(UseCaseError::Refused(Refusal::NoSuchObject(
            oid(1).short().to_string()
        )))
    );
}

#[test]
fn a_token_that_sees_part_selects_only_that_part_and_waits_one_item_per_object() {
    let s = work_and_home();
    let token = token_over(&["work"]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    let out = dam
        .edit_each(&picked(&[("kind", "task")]), retitled("x"), run())
        .unwrap();
    assert_eq!(out.oids.len(), 2);
    assert!(out.changes.iter().all(|c| c.sent.is_some()));
    assert_eq!(s.store.items().unwrap().len(), 2);
    assert_eq!(subject_of(&s, 1), "report");
}

#[test]
fn a_token_that_sees_all_sends_an_item_for_each() {
    let s = work_and_home();
    let token = token_over(&["work", "home"]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    let out = dam
        .remove_each(&picked(&[("kind", "task")]), run())
        .unwrap();
    assert_eq!(out.oids.len(), 3);
    assert_eq!(s.store.items().unwrap().len(), 3);
}

#[test]
fn a_token_preview_sends_no_item() {
    let s = work_and_home();
    let token = token_over(&["work"]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    let previewing = SetRun {
        preview: true,
        ..run()
    };
    let out = dam
        .edit_each(&picked(&[("kind", "task")]), retitled("x"), previewing)
        .unwrap();
    assert_eq!(out.changes.len(), 2);
    assert!(s.store.items().unwrap().is_empty());
}

#[test]
fn a_token_cannot_move_a_set() {
    let s = work_and_home();
    let token = token_over(&["work"]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    assert_eq!(
        dam.relocate_each(&named(&[1]), None, Some(&oid(2)), run()),
        Err(UseCaseError::Refused(Refusal::NotOpenToTokens("mv")))
    );
}

#[test]
fn a_token_change_never_shows_an_object_the_token_cannot_see() {
    let s = work_and_home();
    let mut report = task_in(1, "report", "work");
    report.base.depends = vec![oid(3)];
    report.base.parent = Some(oid(3));
    report.event = Some(oid(3));
    s.store.put(&Object::Task(report)).unwrap();
    let token = token_over(&["work"]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    let out = dam.edit_each(&named(&[1]), retitled("x"), run()).unwrap();
    let shown = &out.changes[0].change;
    for object in shown.before().into_iter().chain(shown.after()) {
        assert!(object.base().depends.is_empty());
        assert_eq!(object.base().parent, None);
        assert_eq!(object.as_task().unwrap().event, None);
    }
}
