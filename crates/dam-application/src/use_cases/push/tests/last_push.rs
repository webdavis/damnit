use crate::errors::UseCaseError;
use crate::ports::{AttemptRepository, Clock, StoreError};
use crate::testing::prelude::*;
use crate::testing::{
    FaultyStore, Scenario, accepting_all, oid, pushing, refusing_all, remotes_launched_by, task,
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
fn a_record_of_the_attempt_that_cannot_be_written_stops_the_push_before_the_call() {
    let s = Scenario::default();
    s.committed(task(1));
    s.store
        .set_accepted_kinds(&todoist(), &task_caps().kinds)
        .unwrap();
    let store = FaultyStore::new(s.store.clone());
    store.fail_at_write(0);
    let (l, pushed) = pushing(accepting_all(Some("r1")));
    let err = s.fixture.dam(&store).push(remotes_launched_by(&l), None);
    assert!(
        matches!(err, Err(UseCaseError::Store(StoreError::Busy(_)))),
        "{err:?}"
    );
    assert!(pushed.borrow().is_empty());
    assert_eq!(store.inner, s.store);
}

#[test]
fn push_bookkeeping_that_fails_midway_records_nothing_but_the_attempt() {
    let s = Scenario::default();
    s.committed(task(1));
    s.store
        .set_accepted_kinds(&todoist(), &task_caps().kinds)
        .unwrap();
    let (l, _) = pushing(accepting_all(Some("r1")));
    for failing in 1.. {
        let store = FaultyStore::new(s.store.clone());
        store.fail_at_write(failing);
        match s.fixture.dam(&store).push(remotes_launched_by(&l), None) {
            Ok(_) => return assert!(failing > 1, "the push settled nothing"),
            Err(UseCaseError::Store(StoreError::Busy(_))) => {}
            Err(other) => panic!("write {failing}: {other:?}"),
        }
        let recorded = store.inner.attempts(&todoist()).unwrap();
        assert_eq!(recorded.keys().collect::<Vec<_>>(), vec![&oid(1)]);
        store.inner.set_attempts(&todoist(), &oid(1), &[]).unwrap();
        assert_eq!(store.inner, s.store, "write {failing} failed");
    }
}
