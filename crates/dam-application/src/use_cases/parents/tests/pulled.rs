use dam_domain::{Object, Path};

use super::{errands, parent_of};
use crate::ports::Conflict;
use crate::ports::{ConflictRepository, ObjectRepository};
use crate::remote::{IncomingObject, PullOutcome};
use crate::testing::{Scenario, oid, pulling, remotes_launched_by, task_in, todoist};
use crate::use_cases::resolve::Side;

pub(super) fn pull(s: &Scenario, object: Object) {
    let l = pulling(PullOutcome {
        objects: vec![IncomingObject {
            remote_id: "r1".into(),
            object,
        }],
        ..PullOutcome::default()
    });
    s.dam().pull(remotes_launched_by(&l), None).unwrap();
}

fn milk_in(path: &str) -> Object {
    Object::Task(task_in(0, "milk", path))
}

#[test]
fn a_pulled_object_under_a_known_holder_takes_it_as_parent() {
    let s = errands();
    pull(&s, milk_in("Home/errands"));
    let milk = s
        .store
        .all()
        .unwrap()
        .into_iter()
        .find(|o| o.base().subject == "milk")
        .unwrap();
    assert_eq!(milk.base().parent, Some(oid(1)));
}

#[test]
fn a_pull_does_not_wipe_a_local_parent() {
    let s = errands();
    let mut milk = task_in(2, "milk", "Home/errands");
    milk.base.parent = Some(oid(1));
    s.committed(Object::Task(milk)).tracked_as(&oid(2), "r1");
    let mut theirs = task_in(0, "milk", "Home/errands");
    theirs.base.body = "oat".into();
    pull(&s, Object::Task(theirs));
    assert_eq!(parent_of(&s, &oid(2)), Some(oid(1)));
}

#[test]
fn a_restored_object_takes_the_holder_of_its_committed_path() {
    let s = errands();
    s.committed(Object::Task(task_in(2, "milk", "Home/errands")));
    s.dam()
        .relocate(&oid(2), Some(&Path::root()), None)
        .unwrap();
    s.dam().restore(&[oid(2)]).unwrap();
    assert_eq!(parent_of(&s, &oid(2)), Some(oid(1)));
}

#[test]
fn resolving_with_theirs_takes_the_holder_of_their_path() {
    let s = errands();
    s.filed(task_in(2, "milk", ""));
    s.store
        .mark_conflict(&Conflict {
            remote: todoist(),
            oid: oid(2),
            ours: Some(Object::Task(task_in(2, "milk", ""))),
            theirs: Object::Task(task_in(2, "milk", "Home/errands")),
        })
        .unwrap();
    s.dam().resolve(&oid(2), Side::Theirs).unwrap();
    assert_eq!(parent_of(&s, &oid(2)), Some(oid(1)));
}
