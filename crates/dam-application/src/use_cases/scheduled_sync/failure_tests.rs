use std::cell::Cell;
use std::rc::Rc;

use dam_domain::{Object, Task};

use super::tests::{sync, syncing_every};
use super::*;
use crate::config::RemoteConfig;
use crate::ports::{Conflict, HelperError, Notice};
use crate::testing::prelude::*;
use crate::testing::{
    ScriptedHelper, ScriptedLauncher, answering, oid, pulling, remotes_launched_by, task, todoist,
};

fn failing_with(why: &str) -> ScriptedLauncher {
    answering(Err(HelperError::Io(why.into())))
}

fn sync_failures(s: &crate::testing::Scenario) -> Vec<Notice> {
    s.store
        .notices()
        .unwrap()
        .into_iter()
        .filter(|n| matches!(n, Notice::SyncFailed { .. }))
        .collect()
}

#[test]
fn a_failed_sync_leaves_a_notice_naming_the_remote_and_why() {
    let s = syncing_every(15);
    let reports = sync(&s, &failing_with("down"));
    let SyncOutcome::Failed { why } = &reports[0].outcome else {
        panic!("{reports:?}");
    };
    assert!(why.contains("down"), "{why}");
    assert_eq!(
        sync_failures(&s),
        vec![Notice::SyncFailed {
            remote: todoist(),
            why: why.clone()
        }]
    );
}

#[test]
fn a_second_failure_replaces_the_first_notice() {
    let s = syncing_every(15);
    sync(&s, &failing_with("down"));
    sync(&s, &failing_with("still down"));
    let notices = sync_failures(&s);
    assert_eq!(notices.len(), 1, "{notices:?}");
    assert!(format!("{notices:?}").contains("still down"), "{notices:?}");
}

#[test]
fn a_sync_that_works_clears_the_notice() {
    let s = syncing_every(15);
    sync(&s, &failing_with("down"));
    sync(&s, &pulling(Default::default()));
    assert_eq!(sync_failures(&s), vec![]);
}

#[test]
fn one_remote_failing_does_not_stop_the_next() {
    let mut s = syncing_every(15);
    let mut work: RemoteConfig = s.fixture.config.remotes[0].clone();
    work.name = crate::ports::RemoteName("work".into());
    s.fixture.config.remotes.push(work);
    let launched = Rc::new(Cell::new(0));
    let launcher = ScriptedLauncher::launching(move || {
        launched.set(launched.get() + 1);
        match launched.get() {
            1 => ScriptedHelper::answering(Err(HelperError::Io("down".into()))),
            _ => ScriptedHelper::answering(Ok(Default::default())),
        }
    });
    let reports = sync(&s, &launcher);
    assert!(
        matches!(
            reports.iter().map(|r| &r.outcome).collect::<Vec<_>>()[..],
            [SyncOutcome::Failed { .. }, SyncOutcome::Synced { .. }]
        ),
        "{reports:?}"
    );
}

#[test]
fn an_open_conflict_holds_that_object_and_the_sync_still_works() {
    let s = syncing_every(15);
    s.committed(task(1));
    s.store
        .mark_conflict(&Conflict {
            remote: todoist(),
            oid: oid(1),
            ours: Some(task(1)),
            theirs: Object::Task(Task::new(oid(1), "theirs")),
        })
        .unwrap();
    let reports = sync(&s, &pulling(Default::default()));
    let SyncOutcome::Synced {
        pushed: Some(push), ..
    } = &reports[0].outcome
    else {
        panic!("{reports:?}");
    };
    assert_eq!((push.sent, push.held), (0, Some(1)));
    assert_eq!(sync_failures(&s), vec![]);
}

#[test]
fn a_cancelled_sync_stops_the_run_and_leaves_no_notice() {
    let s = syncing_every(15);
    let launcher = answering(Err(HelperError::Cancelled));
    let stopped = s
        .dam()
        .sync_due_remotes(remotes_launched_by(&launcher), |e| format!("{e:?}"));
    assert!(
        matches!(stopped, Err(UseCaseError::Helper(HelperError::Cancelled))),
        "{stopped:?}"
    );
    assert_eq!(s.store.notices().unwrap(), vec![]);
}
