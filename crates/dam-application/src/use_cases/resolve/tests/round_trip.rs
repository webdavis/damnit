use super::super::*;
use crate::remote::{IncomingObject, PullOutcome};
use crate::testing::prelude::*;
use crate::testing::{
    Scenario, ScriptedLauncher, accepting_all, oid, pulling, pushing, remotes_launched_by, todoist,
};
use dam_domain::{Object, Task};

fn sending_r1_as_theirs() -> ScriptedLauncher {
    pulling(PullOutcome {
        objects: vec![IncomingObject {
            remote_id: "r1".into(),
            object: Object::Task(Task::new(oid(0), "theirs")),
        }],
        ..PullOutcome::default()
    })
}

fn both_sides_moved_off_one_base() -> Scenario {
    let s = Scenario::default().drawing_from(42);
    s.committed(Object::Task(Task::new(oid(1), "base")))
        .tracked_as(&oid(1), "r1")
        .committed(Object::Task(Task::new(oid(1), "mine")));
    let reports = s
        .dam()
        .pull(remotes_launched_by(&sending_r1_as_theirs()), None)
        .unwrap();
    assert_eq!(reports[0].conflicts, 1);
    s
}

fn pulled_again_before_the_resolution_reached_the_remote(s: &Scenario) -> usize {
    let reports = s
        .dam()
        .pull(remotes_launched_by(&sending_r1_as_theirs()), None)
        .unwrap();
    reports[0].conflicts
}

#[test]
fn ours_is_committed_and_the_conflict_is_not_raised_again() {
    let s = both_sides_moved_off_one_base();
    s.dam().resolve(&oid(1), Side::Ours).unwrap();

    let owed = s.store.unpushed(&todoist()).unwrap();
    let resolution = owed
        .iter()
        .find(|c| c.message.contains("with ours"))
        .expect("the resolution is a commit of its own");
    assert_eq!(
        resolution.changes[0].after().unwrap().base().subject,
        "mine"
    );

    assert_eq!(
        pulled_again_before_the_resolution_reached_the_remote(&s),
        0,
        "the conflict was raised again"
    );
    assert_eq!(
        s.store.get(&oid(1)).unwrap().unwrap().base().subject,
        "mine"
    );

    let (l, _) = pushing(accepting_all(Some("r1")));
    let reports = s.dam().push(remotes_launched_by(&l), None).unwrap();
    assert_eq!(reports[0].sent, 1, "ours was sent upstream");
    assert_eq!(
        s.store
            .remote_snapshot(&todoist(), &oid(1))
            .unwrap()
            .unwrap()
            .base()
            .subject,
        "mine",
        "the tracking snapshot moved off theirs"
    );
}

#[test]
fn theirs_leaves_only_theirs_owed_and_settles_the_conflict() {
    let s = both_sides_moved_off_one_base();
    s.dam().resolve(&oid(1), Side::Theirs).unwrap();
    let (l, sent) = pushing(accepting_all(Some("r1")));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    for m in sent.borrow().iter() {
        assert_eq!(
            m.object.as_ref().unwrap().base().subject,
            "theirs",
            "a push after resolving with theirs must not carry ours"
        );
    }
    assert_eq!(pulled_again_before_the_resolution_reached_the_remote(&s), 0);
    assert_eq!(
        s.store.get(&oid(1)).unwrap().unwrap().base().subject,
        "theirs"
    );
}
