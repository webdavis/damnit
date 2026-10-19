use super::super::*;
use crate::testing::prelude::*;
use crate::testing::{Scenario, event, oid, task};
use dam_domain::{Completion, Object, Task};

#[test]
fn undone_reopens_a_completed_task() {
    let s = Scenario::default();
    let mut done = Task::new(oid(1), "t1");
    done.completion = Completion::Done {
        at: Some(jiff::Timestamp::UNIX_EPOCH),
    };
    s.working(Object::Task(done));
    let id = oid(1);
    let dam = s.dam();
    let out = dam
        .edit(
            &id,
            EditFields {
                undone: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(out.as_task().unwrap().completion, Completion::Open);
    assert_eq!(s.store.get(&id).unwrap(), Some(out));
}

#[test]
fn undone_on_a_task_that_is_not_completed_is_refused_naming_it() {
    let s = Scenario::default();
    s.working(task(1));
    let id = oid(1);
    let dam = s.dam();
    let err = dam
        .edit(
            &id,
            EditFields {
                undone: true,
                ..Default::default()
            },
        )
        .unwrap_err();
    assert_eq!(
        err,
        UseCaseError::Refused(Refusal::NotCompleted(id.clone()))
    );
}

#[test]
fn undone_on_an_event_is_refused_as_not_a_task() {
    let s = Scenario::default();
    s.working(event(9));
    let dam = s.dam();
    let err = dam
        .edit(
            &oid(9),
            EditFields {
                undone: true,
                ..Default::default()
            },
        )
        .unwrap_err();
    assert_eq!(err, UseCaseError::Refused(Refusal::NotATask(oid(9))));
}
