use crate::testing::prelude::*;
use crate::testing::{FaultyStore, Scenario, oid, task_in};
use dam_domain::{Event, Object, Task, When};
use jiff::civil::date;

fn depending_on(byte: u8, subject: &str, on: u8) -> Task {
    let mut t = task_in(byte, subject, "");
    t.base.depends.push(oid(on));
    t
}

#[test]
fn plan_lists_children_dependents_and_attached() {
    let s = Scenario::default();
    s.filed(task_in(1, "plan", ""))
        .filed(task_in(2, "step", "plan"))
        .filed(task_in(3, "detail", "plan/step"))
        .filed(depending_on(4, "dependent", 1));
    let found = s.dam().plan_remove(&oid(1)).unwrap();
    assert_eq!(found.moved_up, vec![oid(2)]);
    assert_eq!(found.dependents, vec![oid(4)]);
}

#[test]
fn remove_lists_only_the_children_it_moved_up_and_keeps_a_grandchild_under_its_parent() {
    let s = Scenario::default();
    s.filed(task_in(1, "plan", "work"))
        .filed(task_in(2, "step", "work/plan"))
        .filed(task_in(3, "note", "work/plan"))
        .filed(task_in(4, "detail", "work/plan/step"));
    let removed = s.dam().remove(&oid(1)).unwrap();
    assert_eq!(removed.moved_up, vec![oid(2), oid(3)]);
    assert_eq!(s.path_of(&oid(2)), "work/");
    assert_eq!(s.path_of(&oid(3)), "work/");
    assert_eq!(s.path_of(&oid(4)), "work/step/");
}

#[test]
fn remove_moves_children_up_and_drops_dependency_edges() {
    let s = Scenario::default();
    s.filed(task_in(1, "plan", "work"))
        .filed(task_in(2, "step", "work/plan"))
        .filed(depending_on(3, "dependent", 1));
    s.dam().remove(&oid(1)).unwrap();
    assert!(s.store.get(&oid(1)).unwrap().is_none());
    assert_eq!(s.path_of(&oid(2)), "work/");
    assert!(
        s.store
            .get(&oid(3))
            .unwrap()
            .unwrap()
            .base()
            .depends
            .is_empty()
    );
}

#[test]
fn removing_a_task_leaves_objects_in_other_folders_alone() {
    let s = Scenario::default();
    s.filed(task_in(1, "plan the offsite", "work"))
        .filed(task_in(2, "unrelated project note", "work/clients"))
        .filed(task_in(3, "another", "work/clients/acme"));
    let plan = s.dam().remove(&oid(1)).unwrap();
    assert!(plan.moved_up.is_empty());
    assert_eq!(s.path_of(&oid(2)), "work/clients/");
    assert_eq!(s.path_of(&oid(3)), "work/clients/acme/");
}

#[test]
fn an_attached_task_keeps_the_removed_events_oid() {
    let s = Scenario::default();
    let day = When::Day(date(2026, 9, 25));
    let mut prep = Task::new(oid(6), "prep");
    prep.event = Some(oid(5));
    s.working(Object::Event(Event::new(
        oid(5),
        "meeting",
        day.clone(),
        day,
    )))
    .filed(prep);
    let plan = s.dam().remove(&oid(5)).unwrap();
    assert_eq!(plan.attached, vec![oid(6)]);
    let kept = s.store.get(&oid(6)).unwrap().unwrap();
    assert_eq!(kept.as_task().unwrap().event, Some(oid(5)));
}

#[test]
fn rm_that_fails_between_two_children_moves_neither() {
    let s = Scenario::default();
    s.filed(task_in(1, "p", "top"))
        .filed(task_in(2, "a", "top/p"))
        .filed(task_in(3, "b", "top/p"));
    let store = FaultyStore::new(s.store);
    let dam = s.fixture.dam(&store);
    store.assert_unchanged_when_any_write_fails(|| dam.remove(&oid(1)).map(drop));
}

mod take_back;
