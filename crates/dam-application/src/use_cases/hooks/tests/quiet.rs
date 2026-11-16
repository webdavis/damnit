use std::collections::BTreeSet;

use dam_domain::Permission::{TaskChange, TaskCreate};
use dam_domain::{CallerId, Object, Path, When};
use jiff::civil::date;

use super::{every_hook, new_task, ran, retitled};
use crate::ports::Conflict;
use crate::service::Dam;
use crate::testing::prelude::*;
use crate::testing::{allowed, holder, oid, task_in, todoist};
use crate::use_cases::items::land::Landed;
use crate::use_cases::new_object::NewEvent;
use crate::use_cases::prefix::Named;
use crate::use_cases::resolve::Side;

#[test]
fn a_tokens_own_command_runs_no_hook() {
    let s = every_hook();
    let tool = holder(vec![allowed(&[TaskCreate, TaskChange], None)]);
    let dam = Dam {
        access: &tool,
        ..s.dam()
    };
    dam.new_task(new_task("call the dentist")).unwrap();
    dam.edit(&oid(1), retitled("pay the rent")).unwrap();
    assert_eq!(ran(&s), []);
}

#[test]
fn a_new_event_runs_no_hook() {
    let s = every_hook();
    s.dam()
        .new_event(NewEvent {
            parent: None,
            subject: "dentist".into(),
            path: Path::root(),
            start: When::Day(date(2026, 9, 18)),
            end: When::Day(date(2026, 9, 19)),
            labels: BTreeSet::new(),
            body: String::new(),
            id: None,
        })
        .unwrap();
    assert_eq!(ran(&s), []);
}

#[test]
fn rm_and_restore_run_no_hook_for_a_task_that_depended_on_the_one_removed() {
    let s = every_hook();
    let mut pack = task_in(2, "pack", "home");
    pack.base.depends = vec![oid(1)];
    s.committed(Object::Task(pack));
    s.dam().remove(&oid(1)).unwrap();
    s.dam().restore(&[oid(1)]).unwrap();
    assert_eq!(ran(&s), []);
}

#[test]
fn reset_of_an_added_item_runs_no_hook_for_the_task_it_sets_back() {
    let s = every_hook();
    let tool = holder(vec![allowed(&[TaskChange], None)]);
    let dam = Dam {
        access: &tool,
        ..s.dam()
    };
    let Landed::Sent(sent) = dam.edit(&oid(1), retitled("pay the rent")).unwrap() else {
        panic!("a token's edit was written, not sent");
    };
    s.dam().add(&[Named::Item(sent.item.hash.clone())]).unwrap();
    s.fixture.hooks.ran.borrow_mut().clear();
    s.dam().reopen(&sent.item.hash).unwrap();
    assert_eq!(ran(&s), []);
}

#[test]
fn resolve_runs_no_hook() {
    let s = every_hook();
    s.store
        .mark_conflict(&Conflict {
            remote: todoist(),
            oid: oid(1),
            ours: s.store.get(&oid(1)).unwrap(),
            theirs: Object::Task(task_in(1, "pay the rent", "home")),
        })
        .unwrap();
    s.dam().resolve(&oid(1), Side::Theirs).unwrap();
    assert_eq!(ran(&s), []);
}

#[test]
fn revert_runs_no_hook() {
    let s = every_hook();
    s.dam().edit(&oid(1), retitled("pay the rent")).unwrap();
    s.dam().add_all().unwrap();
    s.dam().commit("retitled").unwrap();
    s.fixture.hooks.ran.borrow_mut().clear();
    let newest = s.store.log().unwrap()[0].id.to_string();
    s.dam().revert(&newest, &[]).unwrap();
    assert_eq!(ran(&s), []);
}

#[test]
fn rm_by_id_runs_no_hook_for_a_task_that_depended_on_the_one_removed() {
    let s = every_hook();
    let mut named = new_task("call the dentist");
    named.id = Some(CallerId::parse("dentist").unwrap());
    let dentist = s.dam().new_task(named).unwrap();
    let mut pack = task_in(2, "pack", "home");
    pack.base.depends = vec![dentist.oid().clone()];
    s.committed(Object::Task(pack));
    s.fixture.hooks.ran.borrow_mut().clear();
    s.dam()
        .remove_by_id(&CallerId::parse("dentist").unwrap())
        .unwrap();
    assert_eq!(ran(&s), []);
}
