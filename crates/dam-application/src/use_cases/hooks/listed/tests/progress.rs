use super::*;
use crate::config::HookEvent::TaskDone;
use crate::testing::Said;

#[test]
fn the_run_is_one_step_counted_task_by_task_that_finishes() {
    let s = every_hook();
    s.dam().run_hook_over_listed(TaskDone, None).unwrap();
    assert_eq!(
        *s.fixture.progress.said.borrow(),
        [
            Said::Started("task_done: running task_done tasks".into(), Some(3)),
            Said::Advanced(1),
            Said::Advanced(2),
            Said::Advanced(3),
            Said::Finished,
        ]
    );
}
