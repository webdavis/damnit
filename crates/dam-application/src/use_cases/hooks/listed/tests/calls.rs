use super::*;
use crate::config::HookEvent::{TaskDone, TaskNew, TaskPulled};
use crate::errors::{Refusal, UseCaseError};
use crate::testing::{event, oid};

#[test]
fn the_hook_is_called_once_for_each_listed_task_and_never_for_an_event() {
    let s = every_hook();
    s.committed(event(7));
    let run = s
        .dam()
        .run_hook_over_listed(TaskDone, &Filter::default())
        .unwrap();
    assert_eq!(
        ran(&s),
        [
            (TaskDone, "buy milk".to_string()),
            (TaskDone, "call Ann".to_string()),
            (TaskDone, "pay rent".to_string()),
        ]
    );
    assert_eq!((run.hook, run.ran, run.not_reached), (TaskDone, 3, 0));
}

#[test]
fn a_selection_narrows_the_tasks_the_hook_is_called_for() {
    let s = every_hook();
    s.dam().run_hook_over_listed(TaskNew, &errands()).unwrap();
    assert_eq!(ran(&s), [(TaskNew, "buy milk".to_string())]);
}

#[test]
fn task_pulled_gets_no_before_and_a_new_change_from_the_tracking_remote() {
    let s = every_hook();
    s.mapped_as(&oid(2), "r2");
    let run = s
        .dam()
        .run_hook_over_listed(TaskPulled, &Filter::default())
        .unwrap();
    let calls = s.fixture.hooks.pulled.borrow().clone();
    assert_eq!(calls.len(), 1);
    let (task, before, pull) = &calls[0];
    assert_eq!((task.as_str(), before), ("call Ann", &None));
    assert_eq!(
        (pull.remote.0.as_str(), pull.change.as_str()),
        ("todoist", "new")
    );
    assert_eq!((run.ran, run.not_reached), (1, 2));
}

#[test]
fn a_hook_with_no_file_in_the_hooks_folder_is_refused() {
    let s = hooked(&[(TaskNew, None)], None);
    for missing in [TaskDone, TaskPulled] {
        assert_eq!(
            s.dam().run_hook_over_listed(missing, &Filter::default()),
            Err(UseCaseError::Refused(Refusal::HookNotDeclared(missing)))
        );
    }
    assert!(ran(&s).is_empty());
}

#[test]
fn a_done_task_is_run_over_unless_done_is_selected() {
    let s = every_hook();
    let mut done = crate::testing::task_in(4, "filed taxes", "home");
    done.completion = dam_domain::Completion::Done { at: None };
    s.committed(Object::Task(done));
    let every = s
        .dam()
        .run_hook_over_listed(TaskDone, &Filter::default())
        .unwrap();
    let open = s
        .dam()
        .run_hook_over_listed(TaskDone, &selecting("done", "false"))
        .unwrap();
    assert_eq!((every.ran, open.ran), (4, 3));
}

#[test]
fn an_event_field_is_refused_and_no_hook_runs() {
    let s = every_hook();
    let refused = s
        .dam()
        .run_hook_over_listed(TaskNew, &selecting("start", "today"));
    assert!(matches!(
        refused,
        Err(UseCaseError::Select(crate::SelectError::Defaults(
            dam_domain::DefaultsError::EventFieldInHookRun { field: "start" }
        )))
    ));
    assert!(ran(&s).is_empty());
}
