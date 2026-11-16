use dam_domain::Permission::TaskChange;
use dam_domain::{ItemHash, Object, Outcome};

use super::{every_hook, hooked, raised, ran, retitled};
use crate::config::HookEvent::TaskChanged;
use crate::config::OnFailure;
use crate::errors::{Refusal, UseCaseError};
use crate::ports::{Conflict, HookFailure};
use crate::service::Dam;
use crate::testing::prelude::*;
use crate::testing::{Scenario, allowed, holder, oid, task_in, todoist};
use crate::use_cases::items::land::Landed;
use crate::use_cases::prefix::Named;
use crate::use_cases::resolve::Side;

fn clashing_retitle(s: &Scenario) -> ItemHash {
    let tool = holder(vec![allowed(&[TaskChange], None)]);
    let dam = Dam {
        access: &tool,
        ..s.dam()
    };
    let Landed::Sent(sent) = dam.edit(&oid(1), retitled("pay the rent")).unwrap() else {
        panic!("a token's edit was written, not sent");
    };
    s.working(Object::Task(task_in(1, "pay rent now", "home")));
    sent.item.hash
}

fn settle(s: &Scenario, hash: &ItemHash, side: Side) -> Result<(), UseCaseError> {
    s.dam().settle(&Named::Item(hash.clone()), side).map(drop)
}

fn subject(s: &Scenario) -> String {
    s.store
        .get(&oid(1))
        .unwrap()
        .unwrap()
        .base()
        .subject
        .clone()
}

#[test]
fn theirs_on_a_tokens_item_runs_the_hook_adding_it_runs() {
    let s = every_hook();
    let hash = clashing_retitle(&s);
    settle(&s, &hash, Side::Theirs).unwrap();
    assert_eq!(ran(&s), [(TaskChanged, "pay the rent")]);
}

#[test]
fn ours_on_a_tokens_item_runs_no_hook() {
    let s = every_hook();
    let hash = clashing_retitle(&s);
    settle(&s, &hash, Side::Ours).unwrap();
    assert_eq!(ran(&s), []);
}

#[test]
fn theirs_with_a_failing_hook_that_refuses_changes_nothing() {
    let s = hooked(&[(TaskChanged, Some(OnFailure::Refuse))]);
    let hash = clashing_retitle(&s);
    *s.fixture.hooks.failing.borrow_mut() = Some(raised());
    assert_eq!(
        settle(&s, &hash, Side::Theirs),
        Err(UseCaseError::Refused(Refusal::HookRefused(HookFailure {
            event: TaskChanged,
            failure: raised(),
        })))
    );
    assert!(s.store.item(&hash).unwrap().unwrap().is_waiting());
    assert!(s.store.staged().unwrap().is_empty());
    assert_eq!(subject(&s), "pay rent now");
}

#[test]
fn theirs_with_a_failing_hook_that_warns_takes_the_item() {
    let s = hooked(&[(TaskChanged, Some(OnFailure::Warn))]);
    let hash = clashing_retitle(&s);
    *s.fixture.hooks.failing.borrow_mut() = Some(raised());
    settle(&s, &hash, Side::Theirs).unwrap();
    assert!(matches!(
        s.store.item(&hash).unwrap().unwrap().outcome,
        Outcome::Added(_)
    ));
    assert_eq!(subject(&s), "pay the rent");
    assert_eq!(s.fixture.hook_warnings.warned.borrow().len(), 1);
}

#[test]
fn resolving_a_conflict_with_a_remote_runs_no_hook() {
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
