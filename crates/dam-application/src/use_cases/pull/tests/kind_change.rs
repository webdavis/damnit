use jiff::civil::date;

use crate::ports::Notice;
use crate::remote::{IncomingObject, PullOutcome};
use crate::testing::prelude::*;
use crate::testing::{Scenario, ScriptedLauncher, oid, pulling, remotes_launched_by};
use dam_domain::{Event, Kind, Object, Task, When};

fn mine_tracked_as_r1() -> Scenario {
    let s = Scenario::default();
    s.working(Object::Task(Task::new(oid(1), "mine")))
        .mapped_as(&oid(1), "r1");
    s
}

fn sending_r1_as_an_event() -> ScriptedLauncher {
    pulling(PullOutcome {
        objects: vec![IncomingObject {
            remote_id: "r1".into(),
            object: Object::Event(Event::new(
                oid(1),
                "theirs",
                When::Day(date(2026, 1, 1)),
                When::Day(date(2026, 1, 2)),
            )),
        }],
        ..PullOutcome::default()
    })
}

#[test]
fn a_kind_change_upstream_keeps_ours_and_raises_a_notice() {
    let s = mine_tracked_as_r1();
    let l = sending_r1_as_an_event();
    let reports = s.dam().pull(remotes_launched_by(&l), None).unwrap();
    assert_eq!(reports[0].unchanged, 1);
    assert_eq!(
        s.store.get(&oid(1)).unwrap(),
        Some(Object::Task(Task::new(oid(1), "mine")))
    );
    assert_eq!(
        s.store.notices().unwrap(),
        vec![Notice::KindChanged {
            oid: oid(1),
            ours: Kind::Task,
            theirs: Kind::Event
        }]
    );
}

#[test]
fn a_kind_change_is_noticed_once_however_often_it_is_pulled() {
    let s = mine_tracked_as_r1();
    let l = sending_r1_as_an_event();
    for _ in 0..2 {
        s.dam().pull(remotes_launched_by(&l), None).unwrap();
    }
    assert_eq!(s.store.notices().unwrap().len(), 1);
}
