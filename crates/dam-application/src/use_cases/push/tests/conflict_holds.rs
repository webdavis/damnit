use dam_domain::{Object, Oid};

use super::order::held_upstream;
use crate::PushReport;
use crate::ports::{Conflict, RemoteName};
use crate::remote::MutationOp;
use crate::testing::prelude::*;
use crate::testing::{
    Scenario, accepting_all, oid, pushing, remotes_launched_by, task_in, todoist,
};
use crate::use_cases::resolve::Side;

use MutationOp::{Delete, Update};

fn in_conflict_on(s: &Scenario, remote: RemoteName, byte: u8) {
    let ours = s.store.committed(&oid(byte)).unwrap().unwrap();
    let mut theirs = ours.clone();
    theirs.base_mut().subject = "theirs".into();
    let conflict = Conflict {
        remote,
        oid: oid(byte),
        ours: Some(ours),
        theirs,
    };
    s.store.mark_conflict(&conflict).unwrap();
}

fn push(s: &Scenario) -> (PushReport, Vec<(Oid, MutationOp)>) {
    let (l, pushed) = pushing(accepting_all(None));
    let mut reports = s.dam().push(remotes_launched_by(&l), None).unwrap();
    let sent = pushed
        .borrow()
        .iter()
        .map(|m| (m.oid.clone(), m.op))
        .collect();
    (reports.remove(0), sent)
}

fn both_renamed_with_the_first_in_conflict() -> Scenario {
    let s = held_upstream(&[task_in(1, "a", "Home/"), task_in(2, "b", "Home/")]);
    s.filed(task_in(1, "a2", "Home/"))
        .committed(Object::Task(task_in(2, "b2", "Home/")));
    in_conflict_on(&s, todoist(), 1);
    s
}

#[test]
fn an_object_in_conflict_is_held_and_the_rest_is_sent() {
    let s = both_renamed_with_the_first_in_conflict();
    let (report, sent) = push(&s);
    assert_eq!(sent, vec![(oid(2), Update)]);
    assert_eq!(report.held, Some(1));
    assert_eq!(
        s.store.unpushed(&todoist()).unwrap().len(),
        1,
        "the commit that changed the held object stays unpushed"
    );
}

#[test]
fn the_held_object_is_sent_once_resolve_settles_it() {
    let s = both_renamed_with_the_first_in_conflict();
    push(&s);
    s.dam().resolve(&oid(1), Side::Ours).unwrap();
    let (report, sent) = push(&s);
    assert_eq!(sent, vec![(oid(1), Update)]);
    assert_eq!(report.held, Some(0));
    assert!(s.store.unpushed(&todoist()).unwrap().is_empty());
}

#[test]
fn a_conflict_from_another_remote_holds_the_object_here_too() {
    let s = held_upstream(&[task_in(1, "a", "Home/")]);
    s.committed(Object::Task(task_in(1, "a2", "Home/")));
    in_conflict_on(&s, RemoteName("calendar".into()), 1);
    let (report, sent) = push(&s);
    assert!(sent.is_empty());
    assert_eq!(report.held, Some(1));
}

#[test]
fn a_container_is_not_removed_while_an_object_under_it_is_held() {
    let s = held_upstream(&[task_in(1, "P", "Home/"), task_in(2, "c", "Home/P/")]);
    s.filed(task_in(2, "c", "Home/")).deleted(&oid(1));
    in_conflict_on(&s, todoist(), 2);
    let (report, sent) = push(&s);
    assert!(!sent.contains(&(oid(1), Delete)), "{sent:?}");
    assert_eq!(report.held, Some(2));
    assert!(s.store.remote_id(&todoist(), &oid(1)).unwrap().is_some());
}

#[test]
fn an_object_moved_into_a_held_container_is_held() {
    let s = held_upstream(&[task_in(1, "P", "Home/"), task_in(2, "c", "Home/")]);
    s.filed(task_in(1, "Q", "Home/"))
        .committed(Object::Task(task_in(2, "c", "Home/Q/")));
    in_conflict_on(&s, todoist(), 1);
    let (report, sent) = push(&s);
    assert!(sent.is_empty(), "{sent:?}");
    assert_eq!(report.held, Some(2));
}

#[test]
fn an_edit_beside_a_held_container_is_sent() {
    let s = held_upstream(&[task_in(1, "P", "Home/"), task_in(2, "c", "Home/P/")]);
    let mut noted = task_in(1, "P", "Home/");
    noted.base.body = "a note".into();
    s.filed(noted)
        .committed(Object::Task(task_in(2, "c2", "Home/P/")));
    in_conflict_on(&s, todoist(), 1);
    let (report, sent) = push(&s);
    assert_eq!(sent, vec![(oid(2), Update)]);
    assert_eq!(report.held, Some(1));
}

#[test]
fn a_task_never_changed_here_is_not_counted_as_held() {
    let s = held_upstream(&[task_in(1, "a", "Home/")]);
    in_conflict_on(&s, todoist(), 1);
    let (report, sent) = push(&s);
    assert!(sent.is_empty());
    assert_eq!(report.held, Some(0));
}
