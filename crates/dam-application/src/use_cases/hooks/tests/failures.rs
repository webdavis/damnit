use dam_domain::Object;

use super::{hooked, new_task, raised, retitled};
use crate::config::HookEvent::{TaskChanged, TaskNew};
use crate::config::OnFailure;
use crate::errors::{Refusal, UseCaseError};
use crate::ports::HookFailure;
use crate::testing::oid;
use crate::testing::prelude::*;

#[test]
fn a_failing_hook_that_warns_keeps_the_change_and_warns_once() {
    let s = hooked(&[(TaskNew, Some(OnFailure::Warn))]);
    *s.fixture.hooks.failing.borrow_mut() = Some(raised());
    let landed = s.dam().new_task(new_task("call the dentist")).unwrap();
    assert!(s.store.get(landed.oid()).unwrap().is_some());
    assert_eq!(
        *s.fixture.hook_warnings.warned.borrow(),
        [HookFailure {
            event: TaskNew,
            failure: raised(),
        }]
    );
}

#[test]
fn a_failing_hook_that_refuses_changes_nothing_and_names_itself() {
    let s = hooked(&[(TaskChanged, Some(OnFailure::Refuse))]);
    *s.fixture.hooks.failing.borrow_mut() = Some(raised());
    assert_eq!(
        s.dam().edit(&oid(1), retitled("pay the rent")),
        Err(UseCaseError::Refused(Refusal::HookRefused(HookFailure {
            event: TaskChanged,
            failure: raised(),
        })))
    );
    let Some(Object::Task(task)) = s.store.get(&oid(1)).unwrap() else {
        panic!("the task is gone");
    };
    assert_eq!(task.base.subject, "pay rent");
    assert!(s.fixture.hook_warnings.warned.borrow().is_empty());
}
