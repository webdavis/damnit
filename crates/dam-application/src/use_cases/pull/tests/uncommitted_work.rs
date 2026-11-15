use dam_domain::{Object, Path, Task};

use super::super::*;
use super::incoming;
use crate::remote::{IncomingObject, PullOutcome};
use crate::testing::prelude::*;
use crate::testing::{Scenario, oid, pulling, remotes_launched_by, todoist};
use crate::use_cases::prefix::Named;

fn task(subject: &str) -> Task {
    Task::new(oid(1), subject)
}

fn tracked_base() -> Scenario {
    let s = Scenario::default().drawing_from(42);
    s.committed(Object::Task(task("base")))
        .tracked_as(&oid(1), "r1");
    s
}

fn with_body(subject: &str, body: &str) -> IncomingObject {
    let mut pulled = incoming(subject, "r1");
    pulled.object.base_mut().body = body.into();
    pulled
}

fn pull(s: &Scenario, theirs: IncomingObject) -> PullReport {
    let l = pulling(PullOutcome {
        objects: vec![theirs, incoming("brand new", "r2")],
        ..PullOutcome::default()
    });
    s.dam()
        .pull(remotes_launched_by(&l), None)
        .unwrap()
        .remove(0)
}

fn here(s: &Scenario) -> Option<Object> {
    s.store.get(&oid(1)).unwrap()
}

fn assert_held_as_a_conflict(s: &Scenario, report: &PullReport, ours: Option<Object>) {
    assert_eq!(report.conflicts, 1);
    let conflicts = s.store.conflicts().unwrap();
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].ours, ours);
    assert_eq!(here(s), ours, "the pull left the working copy as it was");
    assert_eq!(
        s.store.committed(&oid(1)).unwrap(),
        Some(Object::Task(task("base")))
    );
    assert!(
        s.store
            .oid_for_remote_id(&todoist(), "r2")
            .unwrap()
            .is_some(),
        "every other object still lands"
    );
}

fn assert_left_alone(s: &Scenario, report: &PullReport, ours: Option<Object>) {
    assert_eq!(report.conflicts, 0);
    assert!(s.store.conflicts().unwrap().is_empty());
    assert_eq!(report.created, 1, "only the brand new task is created");
    assert_eq!(here(s), ours);
}

#[test]
fn an_uncommitted_edit_meets_a_change_to_the_same_field_as_a_conflict() {
    let s = tracked_base();
    let mine = Object::Task(task("mine"));
    s.working(mine.clone());
    let report = pull(&s, incoming("theirs", "r1"));
    assert_held_as_a_conflict(&s, &report, Some(mine));
}

#[test]
fn an_uncommitted_edit_meets_a_change_to_another_field_as_a_conflict() {
    let s = tracked_base();
    let mine = Object::Task(task("mine"));
    s.working(mine.clone());
    let report = pull(&s, with_body("base", "theirs"));
    assert_held_as_a_conflict(&s, &report, Some(mine));
}

#[test]
fn an_uncommitted_edit_is_left_alone_when_the_remote_did_not_change() {
    let s = tracked_base();
    let mine = Object::Task(task("mine"));
    s.working(mine.clone());
    let report = pull(&s, incoming("base", "r1"));
    assert_left_alone(&s, &report, Some(mine));
}

#[test]
fn an_uncommitted_delete_is_left_alone_when_the_remote_did_not_change() {
    let s = tracked_base();
    s.store.delete(&oid(1)).unwrap();
    let report = pull(&s, incoming("base", "r1"));
    assert_left_alone(&s, &report, None);
}

#[test]
fn an_uncommitted_delete_meets_a_change_in_the_app_as_a_conflict() {
    let s = tracked_base();
    s.store.delete(&oid(1)).unwrap();
    let report = pull(&s, incoming("theirs", "r1"));
    assert_held_as_a_conflict(&s, &report, None);
}

#[test]
fn a_staged_change_meets_a_change_in_the_app_as_a_conflict() {
    let s = tracked_base();
    let mine = Object::Task(task("mine"));
    s.working(mine.clone());
    s.dam().add(&[Named::Object(oid(1))]).unwrap();
    let report = pull(&s, incoming("theirs", "r1"));
    assert_held_as_a_conflict(&s, &report, Some(mine));
    assert_eq!(s.store.staged().unwrap().len(), 1, "the stage is kept");
}

#[test]
fn a_staged_delete_meets_a_change_in_the_app_as_a_conflict() {
    let s = tracked_base();
    s.store.delete(&oid(1)).unwrap();
    s.dam().add(&[Named::Object(oid(1))]).unwrap();
    let report = pull(&s, incoming("theirs", "r1"));
    assert_held_as_a_conflict(&s, &report, None);
    assert_eq!(s.store.staged().unwrap().len(), 1, "the stage is kept");
}

#[test]
fn an_uncommitted_move_meets_a_change_in_the_app_as_a_conflict() {
    let s = tracked_base();
    let mut moved = task("base");
    moved.base.path = Path::parse("Home/").unwrap();
    let moved = Object::Task(moved);
    s.working(moved.clone());
    let report = pull(&s, incoming("theirs", "r1"));
    assert_held_as_a_conflict(&s, &report, Some(moved));
}

#[test]
fn a_new_object_here_is_left_beside_a_new_object_the_pull_creates() {
    let s = Scenario::default().drawing_from(42);
    let mine = Object::Task(Task::new(oid(1), "brand new"));
    s.working(mine.clone());
    let report = pull(&s, incoming("other", "r1"));
    assert_eq!(report.conflicts, 0);
    assert_eq!(report.created, 2);
    assert_eq!(here(&s), Some(mine));
    assert_eq!(s.store.committed(&oid(1)).unwrap(), None);
}
