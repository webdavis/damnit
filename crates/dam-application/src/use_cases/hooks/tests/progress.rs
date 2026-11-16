use super::{hooked, new_task, raised, retitled};
use crate::config::HookEvent::{TaskChanged, TaskNew};
use crate::config::OnFailure;
use crate::testing::{Said, oid};

#[test]
fn a_task_new_hook_is_a_step_that_finishes() {
    let s = hooked(&[(TaskNew, None)]);
    s.dam().new_task(new_task("call the dentist")).unwrap();
    assert_eq!(
        *s.fixture.progress.said.borrow(),
        [
            Said::Started("task_new: running task_new ".into(), None),
            Said::Finished
        ]
    );
}

#[test]
fn a_failing_task_changed_hook_ends_its_step_as_failed() {
    let s = hooked(&[(TaskChanged, Some(OnFailure::Warn))]);
    *s.fixture.hooks.failing.borrow_mut() = Some(raised());
    s.dam().edit(&oid(1), retitled("pay the rent")).unwrap();
    assert_eq!(
        *s.fixture.progress.said.borrow(),
        [
            Said::Started("task_changed: running task_changed ".into(), None),
            Said::Failed
        ]
    );
}
