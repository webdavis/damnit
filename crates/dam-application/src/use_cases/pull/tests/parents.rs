use dam_domain::{Object, Path, Task};

use super::super::*;
use crate::remote::{IncomingObject, PullOutcome};
use crate::testing::prelude::*;
use crate::testing::{Scenario, oid, pulling, remotes_launched_by, todoist};

fn at(byte: u8, subject: &str, path: &str) -> Object {
    let mut task = Task::new(oid(byte), subject);
    task.base.path = Path::parse(path).unwrap();
    Object::Task(task)
}

fn tracked(s: &Scenario, object: Object, remote_id: &str) {
    let oid = object.oid().clone();
    s.committed(object).tracked_as(&oid, remote_id);
}

fn pull(s: &Scenario, pulled: &[(&str, &str, &str, &str)]) -> PullReport {
    let l = pulling(PullOutcome {
        objects: pulled
            .iter()
            .map(|(remote_id, subject, path, _)| IncomingObject {
                remote_id: remote_id.to_string(),
                object: at(0, subject, path),
            })
            .collect(),
        container_remote_ids: pulled
            .iter()
            .filter(|(.., container)| !container.is_empty())
            .map(|(remote_id, _, _, container)| (remote_id.to_string(), container.to_string()))
            .collect(),
        ..PullOutcome::default()
    });
    s.dam()
        .pull(remotes_launched_by(&l), None)
        .unwrap()
        .remove(0)
}

fn landed(s: &Scenario, remote_id: &str) -> Object {
    let oid = s
        .store
        .oid_for_remote_id(&todoist(), remote_id)
        .unwrap()
        .unwrap();
    s.store.get(&oid).unwrap().unwrap()
}

fn two_errands(s: &Scenario) {
    tracked(s, at(1, "Home", ""), "p:1");
    tracked(s, at(2, "errands", "Home/"), "s:2");
    tracked(s, at(3, "errands", "Home/"), "s:3");
}

#[test]
fn two_same_named_sections_each_keep_their_own_task() {
    let s = Scenario::default().drawing_from(42);
    two_errands(&s);
    pull(
        &s,
        &[
            ("i:milk", "milk", "Home/errands/", "s:2"),
            ("i:stamps", "stamps", "Home/errands/", "s:3"),
        ],
    );
    assert_eq!(landed(&s, "i:milk").base().parent, Some(oid(2)));
    assert_eq!(landed(&s, "i:stamps").base().parent, Some(oid(3)));
}

#[test]
fn a_parent_and_child_created_by_one_pull_meet_in_either_order() {
    let parent = ("s:1", "errands", "", "");
    let child = ("i:milk", "milk", "errands/", "s:1");
    for order in [[child, parent], [parent, child]] {
        let s = Scenario::default().drawing_from(42);
        pull(&s, &order);
        assert_eq!(
            landed(&s, "i:milk").base().parent,
            Some(landed(&s, "s:1").oid().clone()),
            "{order:?}"
        );
    }
}

#[test]
fn a_container_dam_does_not_track_falls_back_to_the_nearest_object_by_path() {
    let s = Scenario::default().drawing_from(42);
    tracked(&s, at(1, "errands", ""), "s:1");
    pull(&s, &[("i:milk", "milk", "errands/", "s:unknown")]);
    assert_eq!(landed(&s, "i:milk").base().parent, Some(oid(1)));
}

#[test]
fn a_container_dam_removed_falls_back_to_the_nearest_object_by_path() {
    let s = Scenario::default().drawing_from(42);
    tracked(&s, at(1, "errands", ""), "s:1");
    tracked(&s, at(2, "errands", ""), "s:2");
    s.deleted(&oid(1));
    pull(&s, &[("i:milk", "milk", "errands/", "s:1")]);
    assert_eq!(landed(&s, "i:milk").base().parent, Some(oid(2)));
}

#[test]
fn a_pull_settles_an_unsettled_object_without_making_a_change() {
    let s = Scenario::default().drawing_from(42);
    two_errands(&s);
    tracked(&s, at(4, "milk", "Home/errands/"), "i:4");
    let commits = s.store.log().unwrap().len();
    let report = pull(&s, &[("i:4", "milk", "Home/errands/", "s:2")]);
    let settled = |o: Option<Object>| o.unwrap().base().parent.clone();
    assert_eq!(settled(s.store.get(&oid(4)).unwrap()), Some(oid(2)));
    assert_eq!(settled(s.store.committed(&oid(4)).unwrap()), Some(oid(2)));
    assert_eq!(
        settled(s.store.remote_snapshot(&todoist(), &oid(4)).unwrap()),
        Some(oid(2))
    );
    assert_eq!(s.store.log().unwrap().len(), commits);
    assert!(s.store.conflicts().unwrap().is_empty());
    assert_eq!((report.updated, report.unchanged), (0, 1));
}

#[test]
fn a_pull_settles_an_object_it_has_no_last_seen_copy_of() {
    let s = Scenario::default().drawing_from(42);
    two_errands(&s);
    s.committed(at(4, "milk", "Home/errands/"))
        .mapped_as(&oid(4), "i:4");
    let commits = s.store.log().unwrap().len();
    pull(&s, &[("i:4", "milk", "Home/errands/", "s:2")]);
    let committed = s.store.committed(&oid(4)).unwrap().unwrap();
    assert_eq!(committed.base().parent, Some(oid(2)));
    assert_eq!(s.store.get(&oid(4)).unwrap(), Some(committed));
    assert_eq!(s.store.log().unwrap().len(), commits);
    assert!(s.store.conflicts().unwrap().is_empty());
}

#[test]
fn settling_leaves_uncommitted_work_alone() {
    let s = Scenario::default().drawing_from(42);
    two_errands(&s);
    tracked(&s, at(4, "milk", "Home/errands/"), "i:4");
    s.working(at(4, "oat milk", "Home/errands/"));
    let report = pull(&s, &[("i:4", "milk", "Home/errands/", "s:2")]);
    assert!(s.store.conflicts().unwrap().is_empty());
    let here = s.store.get(&oid(4)).unwrap().unwrap();
    assert_eq!(
        (here.base().subject.as_str(), here.base().parent.clone()),
        ("oat milk", Some(oid(2)))
    );
    assert_eq!(report.conflicts, 0);
}

#[test]
fn a_pulled_task_stays_in_a_parent_whose_rename_is_not_pushed_yet() {
    let s = Scenario::default().drawing_from(42);
    tracked(&s, at(1, "errands", ""), "s:1");
    s.working(at(1, "chores", ""));
    pull(&s, &[("i:milk", "milk", "errands/", "s:1")]);
    let milk = landed(&s, "i:milk");
    assert_eq!(
        (milk.base().path.as_str(), milk.base().parent.clone()),
        ("chores/", Some(oid(1)))
    );
    let seen = s.store.remote_snapshot(&todoist(), milk.oid()).unwrap();
    assert_eq!(seen.unwrap().base().path.as_str(), "errands/");
}
