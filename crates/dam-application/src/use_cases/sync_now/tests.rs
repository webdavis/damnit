use dam_domain::Oid;

use super::*;
use crate::errors::Refusal;
use crate::ports::{Conflict, HelperError};
use crate::remote::MutationOp;
use crate::testing::prelude::*;
use crate::testing::{
    Pushed, Scenario, ScriptedHelper, ScriptedLauncher, accepting_all, oid, pushing,
    remotes_launched_by, task, todoist,
};

fn sync(s: &Scenario) -> (Result<SyncReport, UseCaseError>, Pushed) {
    let (launcher, pushed) = pushing(accepting_all(None));
    let report = s.dam().sync_now(remotes_launched_by(&launcher), "todoist");
    (report, pushed)
}

fn sent(pushed: &Pushed) -> Vec<(Oid, MutationOp)> {
    pushed
        .borrow()
        .iter()
        .map(|m| (m.oid.clone(), m.op))
        .collect()
}

fn in_conflict(s: &Scenario, byte: u8) {
    let ours = s.store.committed(&oid(byte)).unwrap().unwrap();
    let mut theirs = ours.clone();
    theirs.base_mut().subject = "theirs".into();
    s.store
        .mark_conflict(&Conflict {
            remote: todoist(),
            oid: oid(byte),
            ours: Some(ours),
            theirs,
        })
        .unwrap();
}

#[test]
fn a_sync_pulls_then_sends_the_committed_changes() {
    let s = Scenario::default();
    s.committed(task(1));
    let (report, pushed) = sync(&s);
    let report = report.unwrap();
    assert_eq!(
        (report.pulled.remote.clone(), report.pushed.sent),
        (todoist(), 1)
    );
    assert_eq!(sent(&pushed).len(), 1);
    assert!(s.store.last_pull(&todoist()).unwrap().is_some());
    assert!(s.store.unpushed(&todoist()).unwrap().is_empty());
}

#[test]
fn a_sync_with_nothing_to_do_sends_nothing() {
    let s = Scenario::default();
    let (report, pushed) = sync(&s);
    assert_eq!(report.unwrap().pushed.sent, 0);
    assert!(sent(&pushed).is_empty());
}

#[test]
fn an_object_in_conflict_is_left_and_the_rest_is_sent() {
    let s = Scenario::default();
    s.committed(task(1)).committed(task(2));
    in_conflict(&s, 1);
    let (report, pushed) = sync(&s);
    assert_eq!(report.unwrap().pushed.held, Some(1));
    assert_eq!(sent(&pushed), vec![(oid(2), MutationOp::Create)]);
    assert_eq!(s.store.conflicts().unwrap()[0].oid, oid(1));
}

#[test]
fn a_failed_pull_stops_the_sync_before_the_push() {
    let s = Scenario::default();
    s.committed(task(1));
    let launcher = ScriptedLauncher::launching(|| ScriptedHelper {
        push_answer: Box::new(|_| panic!("pushed after a failed pull")),
        ..ScriptedHelper::answering(Err(HelperError::Cancelled))
    });
    let err = s
        .dam()
        .sync_now(remotes_launched_by(&launcher), "todoist")
        .unwrap_err();
    assert_eq!(err, UseCaseError::Helper(HelperError::Cancelled));
    assert_eq!(s.store.unpushed(&todoist()).unwrap().len(), 1);
}

#[test]
fn work_not_yet_committed_is_not_sent() {
    let s = Scenario::default();
    s.working(task(1));
    let (report, pushed) = sync(&s);
    assert_eq!(report.unwrap().pushed.sent, 0);
    assert!(sent(&pushed).is_empty());
}

#[test]
fn an_unknown_remote_is_refused_as_a_pull_refuses_it() {
    let s = Scenario::default();
    let (report, pushed) = {
        let (launcher, pushed) = pushing(accepting_all(None));
        (
            s.dam().sync_now(remotes_launched_by(&launcher), "nope"),
            pushed,
        )
    };
    assert_eq!(
        report.unwrap_err(),
        UseCaseError::Refused(Refusal::NoSuchRemote("nope".into()))
    );
    assert!(sent(&pushed).is_empty());
}
