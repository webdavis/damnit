use super::*;
use crate::config::HookEvent::TaskChanged;
use crate::ports::HookFailure;
use crate::testing::oid;

#[test]
fn a_failure_drops_only_that_tasks_change_and_the_run_goes_on() {
    let s = hooked(&[(TaskChanged, Some(OnFailure::Refuse))], None);
    *s.fixture.hooks.failing.borrow_mut() = Some(raised());
    requesting(&s, "renamed");
    let run = s.dam().run_hook_over_listed(TaskChanged, None).unwrap();
    assert_eq!(subject(&s, &oid(3)), "buy milk");
    assert_eq!(subject(&s, &oid(2)), "renamed");
    assert_eq!(
        (run.ran, run.changed, run.failed, run.not_reached),
        (3, 1, 1, 0)
    );
    assert_eq!(run.error, Some(raised()));
}

#[test]
fn only_the_first_failure_warns_that_its_task_is_left_as_it_was() {
    let s = hooked(&[(TaskChanged, None)], None);
    *s.fixture.hooks.failing_every_call.borrow_mut() = Some(raised());
    let run = s.dam().run_hook_over_listed(TaskChanged, None).unwrap();
    assert_eq!(run.failed, 3);
    assert_eq!(
        *s.fixture.hook_warnings.dropped.borrow(),
        [HookFailure {
            event: TaskChanged,
            failure: raised(),
        }]
    );
    assert!(s.fixture.hook_warnings.warned.borrow().is_empty());
}
