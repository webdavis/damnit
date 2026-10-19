mod children;
mod dispositions;
mod recurrence;

use super::*;
use crate::ports::Clock;
use crate::testing::prelude::*;
use crate::testing::{FixedClock, FixedRandom, Fixture, Scenario, event, oid, task_in};
use dam_domain::{Completion, Task, Tracked};
use jiff::civil::date;

pub(super) fn task_of(s: &Scenario, id: &Oid) -> Task {
    s.store.get(id).unwrap().unwrap().as_task().unwrap().clone()
}

#[test]
fn an_unblocked_task_is_done() {
    let s = Scenario::default().drawing_from(9);
    s.filed(task_in(1, "milk", ""));
    let id = oid(1);
    let dam = s.dam();
    assert!(dam.plan_complete(&id).unwrap().blockers.is_empty());
    assert_eq!(
        dam.complete(&id, Force::No, Tracked::No).unwrap(),
        Completed::Done
    );
    assert!(task_of(&s, &id).completion.is_done());
}

#[test]
fn an_event_is_not_a_task() {
    let s = Scenario::default().drawing_from(9);
    s.working(event(5));
    let err = s
        .dam()
        .complete(&oid(5), Force::No, Tracked::No)
        .unwrap_err();
    assert_eq!(err, UseCaseError::Refused(Refusal::NotATask(oid(5))));
}

#[test]
fn completing_a_task_records_when_it_happened() {
    let s = Scenario::default().drawing_from(1);
    s.filed(task_in(1, "milk", ""));
    let id = oid(1);
    assert_eq!(task_of(&s, &id).completion, Completion::Open);
    s.dam().complete(&id, Force::No, Tracked::No).unwrap();
    assert_eq!(
        task_of(&s, &id).completion,
        Completion::Done {
            at: Some(s.fixture.clock.now())
        }
    );
}

#[test]
fn completing_an_already_completed_task_changes_nothing() {
    let s = Scenario::default().drawing_from(1);
    s.filed(task_in(1, "milk", ""));
    let id = oid(1);
    s.dam().complete(&id, Force::No, Tracked::No).unwrap();
    let after_first = s.store.get(&id).unwrap().unwrap();

    let next_day = Fixture {
        clock: FixedClock(date(2026, 9, 19)),
        random: FixedRandom::new(1),
        ..Fixture::default()
    };
    next_day
        .dam(&s.store)
        .complete(&id, Force::No, Tracked::No)
        .unwrap();
    assert_eq!(
        s.store.get(&id).unwrap().unwrap(),
        after_first,
        "a second done restamped the completion time"
    );
}

#[test]
fn a_dependency_that_no_longer_exists_counts_as_closed() {
    let s = Scenario::default();
    let mut milk = task_in(1, "milk", "");
    milk.base.depends.push(oid(3));
    s.filed(milk);
    let id = oid(1);
    assert!(s.dam().plan_complete(&id).unwrap().blockers.is_empty());
}
