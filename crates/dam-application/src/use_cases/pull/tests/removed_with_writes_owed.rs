use super::unanswered::{named, pull_outcome, push_accepting, push_with_no_answer, subject};
use crate::ports::{AttemptRepository, Notice};
use crate::remote::{MutationOp, PullOutcome};
use crate::testing::prelude::*;
use crate::testing::{FaultyStore, Scenario, oid, pulling, remotes_launched_by, todoist};
use crate::use_cases::last_seen::LastSeen;

fn removed_in_the_app() -> PullOutcome {
    PullOutcome {
        removed: vec!["i:1".into()],
        ..PullOutcome::default()
    }
}

fn renamed_with_no_answer(s: &Scenario) {
    s.committed(named("a")).tracked_as(&oid(1), "i:1").pushed();
    s.committed(named("b"));
    push_with_no_answer(s);
}

fn removed_upstream_notices(s: &Scenario) -> usize {
    let notices = s.store.notices().unwrap();
    notices
        .iter()
        .filter(|n| matches!(n, Notice::RemovedUpstream { .. }))
        .count()
}

#[test]
fn a_task_removed_in_the_app_is_not_sent_back() {
    let s = Scenario::default();
    renamed_with_no_answer(&s);

    pull_outcome(&s, removed_in_the_app());

    assert_eq!(subject(&s), "b");
    assert_eq!(removed_upstream_notices(&s), 1);
    assert_eq!(push_accepting(&s, "i:2"), vec![]);
    assert!(s.store.attempts(&todoist()).unwrap().is_empty());
}

#[test]
fn a_task_removed_in_the_app_with_writes_owed_is_last_seen_as_owing_nothing() {
    let s = Scenario::default();
    renamed_with_no_answer(&s);

    pull_outcome(&s, removed_in_the_app());

    let last_seen = LastSeen::read(&s.store, &todoist(), &oid(1)).unwrap();
    assert_eq!(last_seen, LastSeen::RemovedInTheAppNothingOwed(named("b")));
    assert_eq!(s.store.remote_id(&todoist(), &oid(1)).unwrap(), None);
    assert_eq!(s.store.push_retries(&todoist()).unwrap(), vec![oid(1)]);
}

#[test]
fn an_edit_after_the_removal_sends_it_as_one_create() {
    let s = Scenario::default();
    renamed_with_no_answer(&s);
    pull_outcome(&s, removed_in_the_app());

    s.committed(named("c"));

    let create = (MutationOp::Create, None, Some("c".into()));
    assert_eq!(push_accepting(&s, "i:2"), vec![create]);
}

#[test]
fn dams_own_delete_coming_back_is_no_notice() {
    let s = Scenario::default();
    s.committed(named("a")).tracked_as(&oid(1), "i:1").pushed();
    s.deleted(&oid(1));
    push_with_no_answer(&s);

    pull_outcome(&s, removed_in_the_app());

    assert_eq!(removed_upstream_notices(&s), 0);
    let last_seen = LastSeen::read(&s.store, &todoist(), &oid(1)).unwrap();
    assert_eq!(last_seen, LastSeen::Never);
    assert_eq!(push_accepting(&s, "i:2"), vec![]);
    assert!(s.store.unpushed(&todoist()).unwrap().is_empty());
    assert!(s.store.attempts(&todoist()).unwrap().is_empty());
}

#[test]
fn a_failed_write_during_a_removal_with_writes_owed_changes_nothing() {
    let s = Scenario::default();
    renamed_with_no_answer(&s);
    let store = FaultyStore::new(s.store);
    let dam = s.fixture.dam(&store);
    let l = pulling(removed_in_the_app());
    store.assert_unchanged_when_any_write_fails(|| {
        dam.pull(remotes_launched_by(&l), None).map(drop)
    });
}

#[test]
fn a_delete_here_not_yet_pushed_then_removed_in_the_app_is_no_notice() {
    let s = Scenario::default();
    renamed_with_no_answer(&s);
    s.deleted(&oid(1));

    pull_outcome(&s, removed_in_the_app());

    assert!(s.store.notices().unwrap().is_empty());
    assert_eq!(push_accepting(&s, "i:2"), vec![]);
    assert!(s.store.unpushed(&todoist()).unwrap().is_empty());
}

#[test]
fn a_failed_push_is_no_longer_noted_once_the_task_is_removed_in_the_app() {
    let s = Scenario::default();
    renamed_with_no_answer(&s);
    let failed = |s: &Scenario| {
        let notices = s.store.notices().unwrap();
        notices
            .iter()
            .filter(|n| matches!(n, Notice::PushFailed { .. }))
            .count()
    };
    assert_eq!(failed(&s), 1);

    pull_outcome(&s, removed_in_the_app());

    assert_eq!(failed(&s), 0);
    assert_eq!(removed_upstream_notices(&s), 1);
}
