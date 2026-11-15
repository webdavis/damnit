use super::incoming;
use crate::ports::{MissingTask, Notice};
use crate::remote::PullOutcome;
use crate::testing::prelude::*;
use crate::testing::{Scenario, oid, pulling, remotes_launched_by, todoist};
use dam_domain::{Completion, Object, Task};

const TRACKED: [(u8, &str, &str); 3] = [(1, "milk", "r1"), (2, "eggs", "r2"), (3, "bread", "r3")];

fn milk_eggs_and_bread_tracked() -> Scenario {
    let s = Scenario::default();
    for (byte, subject, remote_id) in TRACKED {
        s.committed(Object::Task(Task::new(oid(byte), subject)))
            .tracked_as(&oid(byte), remote_id);
    }
    s
}

fn pull_listing(s: &Scenario, remote_ids: &[&str], complete: bool) {
    let objects = TRACKED
        .iter()
        .filter(|(_, _, id)| remote_ids.contains(id))
        .map(|(_, subject, id)| incoming(subject, id))
        .collect();
    let l = pulling(PullOutcome {
        objects,
        complete,
        ..PullOutcome::default()
    });
    s.dam().pull(remotes_launched_by(&l), None).unwrap();
}

fn missing(tasks: &[(u8, &str)]) -> Vec<Notice> {
    let tasks = tasks
        .iter()
        .map(|(byte, subject)| MissingTask {
            oid: oid(*byte),
            subject: subject.to_string(),
        })
        .collect();
    vec![Notice::MissingUpstream {
        remote: todoist(),
        tasks,
    }]
}

#[test]
fn open_tasks_a_complete_listing_leaves_out_are_noticed_until_they_come_back() {
    let s = milk_eggs_and_bread_tracked();
    pull_listing(&s, &["r3"], true);
    assert_eq!(
        s.store.notices().unwrap(),
        missing(&[(1, "milk"), (2, "eggs")])
    );
    pull_listing(&s, &["r1", "r3"], true);
    assert_eq!(s.store.notices().unwrap(), missing(&[(2, "eggs")]));
    pull_listing(&s, &["r1", "r2", "r3"], true);
    assert_eq!(s.store.notices().unwrap(), vec![]);
}

#[test]
fn a_listing_that_does_not_say_it_is_complete_notices_nothing() {
    let s = milk_eggs_and_bread_tracked();
    pull_listing(&s, &["r2"], false);
    assert_eq!(s.store.notices().unwrap(), vec![]);
}

#[test]
fn a_task_done_or_changed_here_is_left_out_of_the_notice() {
    let s = milk_eggs_and_bread_tracked();
    let mut milk = Task::new(oid(1), "milk");
    milk.completion = Completion::Done { at: None };
    s.committed(Object::Task(milk))
        .tracked_as(&oid(1), "r1")
        .committed(Object::Task(Task::new(oid(2), "brown eggs")));
    pull_listing(&s, &[], true);
    assert_eq!(s.store.notices().unwrap(), missing(&[(3, "bread")]));
}
