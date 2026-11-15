use dam_domain::Permission::{TaskChange, TaskComplete, TaskCreate, TaskRemove};
use dam_domain::{Force, ItemHash, Object, Outcome, Tracked};

use super::{every_hook, hooked, new_task, raised, ran, retitled};
use crate::access::Holder;
use crate::config::HookEvent::{TaskChanged, TaskDone, TaskNew};
use crate::config::OnFailure;
use crate::errors::{Refusal, UseCaseError};
use crate::ports::HookFailure;
use crate::service::Dam;
use crate::testing::prelude::*;
use crate::testing::{Scenario, allowed, holder, oid, task_in};
use crate::use_cases::items::land::Landed;
use crate::use_cases::prefix::Named;
use crate::use_cases::remove::Removal;

fn tool() -> Holder {
    holder(vec![allowed(
        &[TaskCreate, TaskChange, TaskComplete, TaskRemove],
        None,
    )])
}

fn as_tool<'a>(s: &'a Scenario, tool: &'a Holder) -> Dam<'a> {
    Dam {
        access: tool,
        ..s.dam()
    }
}

fn sent(landed: Landed) -> ItemHash {
    match landed {
        Landed::Sent(sent) => sent.item.hash,
        Landed::Written(object) => panic!("written, not sent: {object:?}"),
    }
}

fn add(s: &Scenario, hash: &ItemHash) -> Result<(), UseCaseError> {
    s.dam().add(&[Named::Item(hash.clone())]).map(drop)
}

#[test]
fn adding_an_item_that_creates_a_task_runs_task_new() {
    let s = every_hook();
    let tool = tool();
    let hash = sent(
        as_tool(&s, &tool)
            .new_task(new_task("call the dentist"))
            .unwrap(),
    );
    add(&s, &hash).unwrap();
    assert_eq!(ran(&s), [(TaskNew, "call the dentist")]);
}

#[test]
fn adding_an_item_that_completes_a_task_runs_task_done() {
    let s = every_hook();
    let tool = tool();
    let (_, landed) = as_tool(&s, &tool)
        .complete(&oid(1), Force::No, Tracked::No)
        .unwrap();
    add(&s, &sent(landed)).unwrap();
    assert_eq!(ran(&s), [(TaskDone, "pay rent")]);
}

#[test]
fn adding_an_item_that_edits_a_task_runs_task_changed() {
    let s = every_hook();
    let tool = tool();
    let hash = sent(
        as_tool(&s, &tool)
            .edit(&oid(1), retitled("pay the rent"))
            .unwrap(),
    );
    add(&s, &hash).unwrap();
    assert_eq!(ran(&s), [(TaskChanged, "pay the rent")]);
}

#[test]
fn adding_a_removal_runs_task_changed_for_the_tasks_it_moved_and_none_for_the_removed() {
    let s = every_hook();
    s.committed(Object::Task(task_in(2, "trip", "home")));
    s.committed(Object::Task(task_in(3, "book flights", "home/trip")));
    let mut pack = task_in(4, "pack", "home");
    pack.base.depends = vec![oid(2)];
    s.committed(Object::Task(pack));
    let tool = tool();
    let Removal::Waiting(waiting) = as_tool(&s, &tool).remove_or_send(&oid(2)).unwrap() else {
        panic!("a token's removal was not sent");
    };
    add(&s, &waiting.item).unwrap();
    assert_eq!(
        ran(&s),
        [(TaskChanged, "book flights"), (TaskChanged, "pack")]
    );
}

#[test]
fn adding_every_working_change_runs_no_hook() {
    let s = every_hook();
    s.dam().edit(&oid(1), retitled("pay the rent")).unwrap();
    s.dam().new_task(new_task("call the dentist")).unwrap();
    s.fixture.hooks.ran.borrow_mut().clear();
    s.dam().add_all().unwrap();
    assert_eq!(ran(&s), []);
}

#[test]
fn a_failing_hook_that_warns_keeps_the_acceptance() {
    let s = hooked(&[(TaskNew, Some(OnFailure::Warn))]);
    let tool = tool();
    let hash = sent(
        as_tool(&s, &tool)
            .new_task(new_task("call the dentist"))
            .unwrap(),
    );
    *s.fixture.hooks.failing.borrow_mut() = Some(raised());
    add(&s, &hash).unwrap();
    assert!(s.store.get(hash.oid()).unwrap().is_some());
    assert!(matches!(
        s.store.item(&hash).unwrap().unwrap().outcome,
        Outcome::Added(_)
    ));
    assert_eq!(s.fixture.hook_warnings.warned.borrow().len(), 1);
}

#[test]
fn a_failing_hook_that_refuses_leaves_the_item_waiting_and_nothing_staged() {
    let s = hooked(&[(TaskChanged, Some(OnFailure::Refuse))]);
    let tool = tool();
    let hash = sent(
        as_tool(&s, &tool)
            .edit(&oid(1), retitled("pay the rent"))
            .unwrap(),
    );
    *s.fixture.hooks.failing.borrow_mut() = Some(raised());
    assert_eq!(
        add(&s, &hash),
        Err(UseCaseError::Refused(Refusal::HookRefused(HookFailure {
            event: TaskChanged,
            failure: raised(),
        })))
    );
    assert!(s.store.item(&hash).unwrap().unwrap().is_waiting());
    assert!(s.store.staged().unwrap().is_empty());
    let task = s.store.get(&oid(1)).unwrap().unwrap();
    assert_eq!(task.base().subject, "pay rent");
}

#[test]
fn adding_every_change_runs_a_hook_for_each_item_it_takes() {
    let s = every_hook();
    let tool = tool();
    sent(
        as_tool(&s, &tool)
            .new_task(new_task("call the dentist"))
            .unwrap(),
    );
    s.dam().add_all().unwrap();
    assert_eq!(ran(&s), [(TaskNew, "call the dentist")]);
}
