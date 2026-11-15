use std::time::Duration;

use jiff::SignedDuration;

use super::*;
use crate::config::ConfiguredDuration;
use crate::remote::{MutationOutcome, MutationVerdict, RemoteCapabilities, RemoteMutation};
use crate::testing::prelude::*;
use crate::testing::{
    Scenario, ScriptedHelper, ScriptedLauncher, config, pulling, pushing, remotes_launched_by,
    task, task_caps, todoist,
};

pub(super) fn syncing_every(minutes: u64) -> Scenario {
    let mut config = config();
    config.remotes[0].sync_every = Some(ConfiguredDuration {
        value: Duration::from_secs(minutes * 60),
        text: format!("{minutes}m"),
    });
    Scenario::default().configured(config)
}

pub(super) fn sync(s: &Scenario, launcher: &ScriptedLauncher) -> Vec<SyncReport> {
    s.dam()
        .sync_due_remotes(remotes_launched_by(launcher), |e| format!("{e:?}"))
        .unwrap()
}

fn launches(launcher: &ScriptedLauncher) -> usize {
    launcher.launched_with.borrow().len()
}

fn accepting(ms: &[RemoteMutation]) -> Vec<MutationOutcome> {
    ms.iter()
        .map(|m| MutationOutcome {
            oid: m.oid.clone(),
            verdict: MutationVerdict::Accepted {
                remote_id: Some(format!("r-{}", m.oid)),
            },
        })
        .collect()
}

#[test]
fn a_remote_never_synced_is_pulled_then_sent_its_committed_changes() {
    let s = syncing_every(15);
    s.committed(task(1));
    let (launcher, pushed) = pushing(accepting);
    let reports = sync(&s, &launcher);
    let [
        SyncReport {
            remote, outcome, ..
        },
    ] = reports.as_slice()
    else {
        panic!("{reports:?}");
    };
    assert_eq!(*remote, todoist());
    let SyncOutcome::Synced {
        pushed: Some(push), ..
    } = outcome
    else {
        panic!("{outcome:?}");
    };
    assert_eq!((push.sent, pushed.borrow().len()), (1, 1));
    assert!(s.store.last_pull(&todoist()).unwrap().is_some());
}

#[test]
fn a_remote_synced_inside_its_interval_is_not_due() {
    let s = syncing_every(15);
    s.pulled_ago(SignedDuration::from_mins(14));
    let launcher = pulling(Default::default());
    let reports = sync(&s, &launcher);
    assert!(
        matches!(reports[0].outcome, SyncOutcome::NotDue),
        "{reports:?}"
    );
    assert_eq!(launches(&launcher), 0);
}

#[test]
fn a_remote_synced_longer_ago_than_its_interval_is_due() {
    let s = syncing_every(15);
    s.pulled_ago(SignedDuration::from_mins(16));
    let reports = sync(&s, &pulling(Default::default()));
    assert!(
        matches!(reports[0].outcome, SyncOutcome::Synced { .. }),
        "{reports:?}"
    );
}

#[test]
fn a_remote_without_sync_every_is_left_out() {
    let s = Scenario::default();
    let launcher = pulling(Default::default());
    assert!(sync(&s, &launcher).is_empty());
    assert_eq!(launches(&launcher), 0);
}

#[test]
fn a_remote_that_takes_no_kinds_gets_the_pull_alone() {
    let s = syncing_every(15);
    s.committed(task(1));
    let launcher = ScriptedLauncher::launching(|| ScriptedHelper {
        caps: RemoteCapabilities {
            kinds: vec![],
            ..task_caps()
        },
        push_answer: Box::new(|_: &[RemoteMutation]| panic!("a read-only remote was pushed to")),
        ..ScriptedHelper::answering(Ok(Default::default()))
    });
    let reports = sync(&s, &launcher);
    assert!(
        matches!(reports[0].outcome, SyncOutcome::Synced { pushed: None, .. }),
        "{reports:?}"
    );
    assert_eq!(launches(&launcher), 1, "only the pull started the helper");
}
