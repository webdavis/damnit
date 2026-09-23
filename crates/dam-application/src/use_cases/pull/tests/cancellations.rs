use std::cell::RefCell;
use std::rc::Rc;

use dam_domain::{EventStatus, Field, Kind, Object, Task, When};
use jiff::civil::date;

use super::*;

fn meeting() -> dam_domain::Event {
    dam_domain::Event::new(
        oid(5),
        "meeting",
        When::Day(date(2026, 9, 25)),
        When::Day(date(2026, 9, 26)),
    )
}

/// The event as it was pulled and committed: working, committed and the
/// remote's snapshot all agree, and "e1" maps to it.
fn pulled(store: &MemoryStore, event: &dam_domain::Event) {
    store.put(&Object::Event(event.clone())).unwrap();
    add_all(store, store, store).unwrap();
    commit(
        store,
        store,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(9),
        "pulled",
    )
    .unwrap();
    store
        .map_remote_id(&remote(), &event.base.oid, "e1")
        .unwrap();
    store
        .set_remote_snapshot(&remote(), &Object::Event(event.clone()))
        .unwrap();
}

fn cancelling(ids: &[&str]) -> ScriptedLauncher {
    declaring(
        ids,
        vec![Field::Subject, Field::Start, Field::End, Field::Status],
    )
}

fn declaring(ids: &[&str], fields: Vec<Field>) -> ScriptedLauncher {
    let mut caps = task_caps();
    caps.kinds = vec![Kind::Event];
    caps.fields = fields;
    let answer = PullOutcome {
        cancelled: ids.iter().map(|s| s.to_string()).collect(),
        ..PullOutcome::default()
    };
    ScriptedLauncher {
        make: Box::new(move || ScriptedHelper {
            caps: caps.clone(),
            pull_answer: answer.clone(),
            push_answer: Box::new(|_| vec![]),
            pushed: Rc::new(RefCell::new(vec![])),
            pulled_since: Rc::new(RefCell::new(vec![])),
        }),
        launched_with: Rc::new(RefCell::new(vec![])),
    }
}

fn run(store: &MemoryStore, ids: &[&str]) -> Result<Vec<PullReport>, UseCaseError> {
    run_with(store, &cancelling(ids))
}

fn run_with(
    store: &MemoryStore,
    launcher: &ScriptedLauncher,
) -> Result<Vec<PullReport>, UseCaseError> {
    pull(
        Repositories::of(store),
        launcher,
        &EchoCredentials,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(42),
        &config(),
        None,
    )
}

#[test]
fn a_cancelled_id_moves_the_event_it_maps_to_cancelled() {
    let store = MemoryStore::new();
    pulled(&store, &meeting());
    let reports = run(&store, &["e1"]).unwrap();
    assert_eq!(reports[0].updated, 1);
    let Some(Object::Event(now)) = store.get(&oid(5)).unwrap() else {
        panic!("the event is gone")
    };
    assert_eq!(now.status, EventStatus::Cancelled);
    assert_eq!(now.base.subject, "meeting", "only the status moved");
    assert_eq!(
        store.oid_for_remote_id(&remote(), "e1").unwrap(),
        Some(oid(5)),
        "still tracked"
    );
}

/// A cancellation lands under the declared-fields rule, so a helper that
/// does not declare `status` cancels nothing.
#[test]
fn a_helper_that_does_not_declare_status_cancels_nothing() {
    let store = MemoryStore::new();
    pulled(&store, &meeting());
    let silent = declaring(&["e1"], vec![Field::Subject, Field::Start, Field::End]);
    let reports = run_with(&store, &silent).unwrap();
    assert_eq!((reports[0].updated, reports[0].unchanged), (0, 1));
    let Some(Object::Event(now)) = store.get(&oid(5)).unwrap() else {
        panic!("the event is gone")
    };
    assert_eq!(now.status, EventStatus::Confirmed);
}

#[test]
fn a_cancelled_event_with_an_attached_task_is_reported_and_the_task_kept() {
    let store = MemoryStore::new();
    pulled(&store, &meeting());
    let mut prep = Task::new(oid(6), "prep");
    prep.event = Some(oid(5));
    store.put(&Object::Task(prep)).unwrap();
    run(&store, &["e1"]).unwrap();
    assert!(matches!(
        store.notices().unwrap().last(),
        Some(Notice::EventCancelled { attached: 1, .. })
    ));
    assert!(store.get(&oid(6)).unwrap().is_some());
}

#[test]
fn an_id_dam_never_mapped_and_one_mapped_to_a_task_change_nothing() {
    let store = MemoryStore::new();
    store
        .put(&Object::Task(Task::new(oid(7), "a task")))
        .unwrap();
    store.map_remote_id(&remote(), &oid(7), "t1").unwrap();
    let reports = run(&store, &["never-seen", "t1"]).unwrap();
    let r = &reports[0];
    assert_eq!(
        (r.created, r.updated, r.unchanged, r.conflicts),
        (0, 0, 0, 0),
        "{r:?}"
    );
    assert!(matches!(store.get(&oid(7)).unwrap(), Some(Object::Task(_))));
}

#[test]
fn an_event_already_cancelled_is_unchanged() {
    let store = MemoryStore::new();
    let mut gone = meeting();
    gone.status = EventStatus::Cancelled;
    pulled(&store, &gone);
    assert_eq!(run(&store, &["e1"]).unwrap()[0].unchanged, 1);
}

#[test]
fn an_uncommitted_local_edit_stops_the_pull_as_any_upstream_change_would() {
    let store = MemoryStore::new();
    pulled(&store, &meeting());
    let mut edited = meeting();
    edited.base.subject = "renamed here".into();
    store.put(&Object::Event(edited)).unwrap();
    assert!(matches!(
        run(&store, &["e1"]).unwrap_err(),
        UseCaseError::Refused(Refusal::DirtyOnPull { .. })
    ));
}

/// A committed local edit meets the cancellation as any upstream change: a
/// conflict whose upstream side is what the remote held, cancelled.
#[test]
fn a_committed_local_edit_and_a_cancellation_conflict_over_what_the_remote_held() {
    let store = MemoryStore::new();
    pulled(&store, &meeting());
    let mut edited = meeting();
    edited.base.subject = "renamed here".into();
    store.put(&Object::Event(edited)).unwrap();
    add_all(&store, &store, &store).unwrap();
    commit(
        &store,
        &store,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(10),
        "renamed",
    )
    .unwrap();
    assert_eq!(run(&store, &["e1"]).unwrap()[0].conflicts, 1);
    let Object::Event(theirs) = &store.conflicts().unwrap()[0].theirs else {
        panic!("not an event")
    };
    assert_eq!(
        (theirs.base.subject.as_str(), theirs.status),
        ("meeting", EventStatus::Cancelled)
    );
}

/// Without a snapshot the working copy stands in, so the cancellation is
/// still raised rather than dropped.
#[test]
fn a_cancellation_for_an_event_with_no_snapshot_is_still_raised() {
    let store = MemoryStore::new();
    store.put(&Object::Event(meeting())).unwrap();
    add_all(&store, &store, &store).unwrap();
    commit(
        &store,
        &store,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(9),
        "pulled",
    )
    .unwrap();
    store.map_remote_id(&remote(), &oid(5), "e1").unwrap();
    assert_eq!(run(&store, &["e1"]).unwrap()[0].conflicts, 1);
}
