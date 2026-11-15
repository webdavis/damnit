use dam_domain::{Author, Path};

use super::edits::requesting;
use super::*;
use crate::ports::{CommitRepository, StageRepository};
use crate::testing::task_in;

#[test]
fn an_edit_is_committed_after_the_pull_naming_the_hook() {
    let s = pulled_hooked();
    tracked(&s, 1, "old");
    s.pushed();
    requesting(&s, "filed");
    let report = pull_objects(&s, vec![incoming("new", "r1")]);
    assert_eq!(committed_subject(&s, &oid(1)), "filed");
    assert!(s.store.staged().unwrap().is_empty());
    let log = s.store.log().unwrap();
    let (hook, pull) = (&log[0], &log[1]);
    assert_eq!(hook.message, "task_pulled after pull from todoist");
    assert!(!hook.pulled && pull.pulled);
    let entry = &hook.changes[0];
    assert_eq!(entry.author, Author::Owner);
    let details = entry.details.as_ref().unwrap();
    assert_eq!(
        details.meta.get("hook").map(String::as_str),
        Some("task_pulled")
    );
    let owed = s.store.unpushed(&todoist()).unwrap();
    assert_eq!(owed.iter().map(|c| &c.id).collect::<Vec<_>>(), [&hook.id]);
    assert_eq!(report.task_pulled.unwrap().commit, Some(hook.id.clone()));
}

#[test]
fn no_commit_is_made_when_no_edit_changed_anything() {
    let s = pulled_hooked();
    requesting(&s, "a");
    let report = pull_objects(&s, vec![incoming("a", "r1")]);
    assert_eq!(s.store.log().unwrap().len(), 1);
    assert_eq!(report.task_pulled.unwrap().commit, None);
}

#[test]
fn the_commit_carries_what_the_edit_moved_but_never_unfinished_work() {
    let s = pulled_hooked();
    tracked(&s, 1, "home");
    s.committed(Object::Task(task_in(5, "pay rent", "home")));
    s.committed(Object::Task(task_in(6, "call Ann", "home")));
    s.working(Object::Task(task_in(6, "call Ann at 5", "home")));
    requesting(&s, "house");
    let mut moved = incoming("home", "r1");
    moved.object.base_mut().body = "rooms".into();
    pull_objects(&s, vec![moved]);
    let path = |oid: &Oid| s.store.committed(oid).unwrap().unwrap().base().path.clone();
    assert_eq!(path(&oid(5)), Path::parse("house").unwrap());
    assert_eq!(path(&oid(6)), Path::parse("home").unwrap());
    assert_eq!(subject(&s, &oid(6)), "call Ann at 5");
}
