use std::cell::RefCell;
use std::rc::Rc;

use jiff::civil::date;

use super::cancellations::{meeting, run};
use super::{config, remote};
use crate::ports::Notice;
use crate::remote::{IncomingObject, PullOutcome};
use crate::service::Dam;
use crate::testing::prelude::*;
use crate::testing::{
    FixedClock, FixedRandom, Fixture, MemoryStore, ScriptedHelper, ScriptedLauncher, oid,
    remotes_launched_by, task_caps,
};
use dam_domain::{Field, Kind, Object, Task};

#[test]
fn a_cancelled_event_that_conflicts_still_raises_the_notice() {
    let store = MemoryStore::new();
    let event = dam_domain::Event::new(
        oid(5),
        "meeting",
        dam_domain::When::Day(date(2026, 1, 1)),
        dam_domain::When::Day(date(2026, 1, 2)),
    );
    store.put(&Object::Event(event.clone())).unwrap();
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    dam.add_all().unwrap();
    Dam {
        random: &FixedRandom::new(20),
        ..dam
    }
    .commit("m")
    .unwrap();
    store.map_remote_id(&remote(), &oid(5), "e1").unwrap();
    store
        .set_remote_snapshot(&remote(), &Object::Event(event.clone()))
        .unwrap();

    let mut edited = event.clone();
    edited.base.subject = "moved".into();
    store.put(&Object::Event(edited)).unwrap();
    dam.add_all().unwrap();
    Dam {
        random: &FixedRandom::new(21),
        ..dam
    }
    .commit("m2")
    .unwrap();

    let mut attached = Task::new(oid(6), "prep");
    attached.event = Some(oid(5));
    store.put(&Object::Task(attached)).unwrap();

    let mut cancelled = event.clone();
    cancelled.status = dam_domain::EventStatus::Cancelled;
    let w = IncomingObject {
        remote_id: "e1".into(),
        object: Object::Event(cancelled),
    };
    let mut caps = task_caps();
    caps.kinds = vec![Kind::Event];
    caps.fields = vec![Field::Subject, Field::Status, Field::Start, Field::End];
    let l = ScriptedLauncher {
        make: Box::new(move || ScriptedHelper {
            caps: caps.clone(),
            pull_answer: Ok(PullOutcome {
                objects: vec![w.clone()],
                ..PullOutcome::default()
            }),
            push_answer: Box::new(|_| Ok(vec![])),
            pushed: Rc::new(RefCell::new(vec![])),
            pulled_since: Rc::new(RefCell::new(vec![])),
        }),
        launched_with: Rc::new(RefCell::new(vec![])),
    };
    let reports = Dam {
        store: &store,
        clock: &FixedClock(date(2026, 9, 18)),
        random: &FixedRandom::new(42),
        config: &config(),
    }
    .pull(remotes_launched_by(&l), None)
    .unwrap();
    assert_eq!(reports[0].conflicts, 1);
    assert_eq!(store.conflicts().unwrap().len(), 1);
    assert!(matches!(
        store.notices().unwrap().last(),
        Some(Notice::EventCancelled { attached: 1, .. })
    ));
}

#[test]
fn a_cancellation_for_an_event_with_no_snapshot_is_raised_against_the_working_copy() {
    let store = MemoryStore::new();
    store.put(&Object::Event(meeting())).unwrap();
    let fixture = Fixture {
        random: FixedRandom::new(9),
        ..Fixture::default()
    };
    let dam = fixture.dam(&store);
    dam.add_all().unwrap();
    dam.commit("pulled").unwrap();
    store.map_remote_id(&remote(), &oid(5), "e1").unwrap();
    assert_eq!(run(&store, &["e1"]).unwrap()[0].conflicts, 1);
}
