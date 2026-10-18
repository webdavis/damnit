use super::super::*;
use super::{put_task, task_of, today};
use crate::testing::prelude::*;
use crate::testing::{FixedClock, FixedRandom, MemoryStore, oid};
use dam_domain::{Completion, Object, Recurrence, Task, When};
use jiff::civil::date;

#[test]
fn a_recurring_task_rolls_forward_instead_of_closing() {
    let store = MemoryStore::new();
    let id = put_task(&store, 1, "water the plants", "", false);
    let mut t = store.get(&id).unwrap().unwrap();
    t.base_mut().recurrence = Some(Recurrence::parse("every week"));
    if let Object::Task(task) = &mut t {
        task.due = Some(When::Day(date(2026, 9, 18)));
    }
    store.put(&t).unwrap();
    let out = complete(
        &store,
        &today(),
        &FixedRandom::new(9),
        &id,
        Force::No,
        Tracked::No,
    )
    .unwrap();
    assert_eq!(
        out,
        Completed::RolledForward {
            next_due: date(2026, 9, 25)
        }
    );
    let t = task_of(&store, &id);
    assert!(!t.completion.is_done());
    assert_eq!(t.due, Some(When::Day(date(2026, 9, 25))));
}

#[test]
fn a_rule_dam_cannot_read_is_refused_and_the_task_stays_open() {
    let store = MemoryStore::new();
    let id = put_task(&store, 1, "water the plants", "", false);
    let mut t = store.get(&id).unwrap().unwrap();
    t.base_mut().recurrence = Some(Recurrence::parse("every other week"));
    store.put(&t).unwrap();
    let refused = |tracked| {
        complete(
            &store,
            &today(),
            &FixedRandom::new(9),
            &id,
            Force::No,
            tracked,
        )
        .unwrap_err()
    };
    assert_eq!(
        refused(Tracked::No),
        UseCaseError::Refused(Refusal::UnreadableRecurrence {
            rule: "every other week".into(),
            tracked: Tracked::No,
        })
    );
    assert_eq!(
        refused(Tracked::No).to_string(),
        "cannot roll \"every other week\" forward; set --recurrence"
    );
    assert_eq!(
        refused(Tracked::ByRemote).to_string(),
        "cannot roll \"every other week\" forward; complete it in Todoist or set --recurrence"
    );
    assert!(!task_of(&store, &id).completion.is_done());
}

#[test]
fn a_task_that_rolls_forward_records_no_completion_time() {
    let store = MemoryStore::new();
    let mut t = Task::new(oid(1), "water the plants");
    t.base.recurrence = Some(Recurrence::parse("every day"));
    t.due = Some(When::Day(date(2026, 9, 18)));
    store.put(&Object::Task(t)).unwrap();
    complete(
        &store,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(1),
        &oid(1),
        Force::No,
        Tracked::No,
    )
    .unwrap();
    let task = task_of(&store, &oid(1));
    assert_eq!(task.completion, Completion::Open);
}
