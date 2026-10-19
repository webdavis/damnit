use crate::ports::Clock;
use crate::testing::prelude::*;
use crate::testing::{
    FaultyStore, Scenario, accepting_all, pushing, refusing_all, remotes_launched_by, task,
    task_caps, todoist,
};

#[test]
fn a_push_that_reached_the_remote_records_when_it_did() {
    let s = Scenario::default();
    assert_eq!(s.store.last_push(&todoist()).unwrap(), None);
    s.committed(task(1));
    let (l, _) = pushing(accepting_all(Some("r1")));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    assert_eq!(
        s.store.last_push(&todoist()).unwrap(),
        Some(s.fixture.clock.now())
    );
}

#[test]
fn a_push_with_nothing_to_send_records_the_time_too() {
    let s = Scenario::default();
    let (l, pushed) = pushing(|_| vec![]);
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    assert!(pushed.borrow().is_empty());
    assert_eq!(
        s.store.last_push(&todoist()).unwrap(),
        Some(s.fixture.clock.now())
    );
}

#[test]
fn a_push_whose_mutations_the_remote_refused_still_records_when_it_did() {
    let s = Scenario::default();
    s.committed(task(1));
    let (l, _) = pushing(refusing_all("rate limited"));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    assert_eq!(
        s.store.last_push(&todoist()).unwrap(),
        Some(s.fixture.clock.now())
    );
}

#[test]
fn push_bookkeeping_that_fails_midway_records_nothing() {
    let s = Scenario::default();
    s.committed(task(1));
    s.store
        .set_accepted_kinds(&todoist(), &task_caps().kinds)
        .unwrap();
    let store = FaultyStore::new(s.store.clone());
    let (l, _) = pushing(accepting_all(Some("r1")));
    store.assert_unchanged_when_any_write_fails(|| {
        s.fixture
            .dam(&store)
            .push(remotes_launched_by(&l), None)
            .map(drop)
    });
}
