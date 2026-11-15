mod committed;
mod details;
mod edits;
mod interrupt;
mod limits;
mod notice;
mod progress;
mod runs;
mod skips;
mod slow;
mod warnings;

use dam_domain::{Completion, Object, Oid, Task};

use crate::config::{Config, PulledChanges, PulledHook};
use crate::ports::{ObjectRepository, RemoteTrackingRepository};
use crate::remote::{IncomingObject, PullOutcome};
use crate::testing::{Scenario, config, oid, pulling, remotes_launched_by, todoist};
use crate::use_cases::pull::PullReport;

fn pulled_hooked() -> Scenario {
    pulled_hooked_leaving(None)
}

fn pulled_hooked_leaving(changes: Option<PulledChanges>) -> Scenario {
    let s = Scenario::default().drawing_from(42).configured(Config {
        task_pulled: Some(PulledHook { changes }),
        ..config()
    });
    s.store.set_sync_token(&todoist(), Some("s0")).unwrap();
    s
}

fn tracked(s: &Scenario, byte: u8, subject: &str) {
    s.committed(Object::Task(Task::new(oid(byte), subject)))
        .tracked_as(&oid(byte), &format!("r{byte}"));
}

fn incoming(subject: &str, remote_id: &str) -> IncomingObject {
    IncomingObject {
        remote_id: remote_id.to_string(),
        object: Object::Task(Task::new(oid(0), subject)),
    }
}

fn done(subject: &str, remote_id: &str) -> IncomingObject {
    let mut pulled = incoming(subject, remote_id);
    if let Object::Task(task) = &mut pulled.object {
        task.completion = Completion::Done { at: None };
    }
    pulled
}

fn pull(s: &Scenario, outcome: PullOutcome) -> PullReport {
    let l = pulling(outcome);
    s.dam()
        .pull(remotes_launched_by(&l), None)
        .unwrap()
        .remove(0)
}

fn pull_objects(s: &Scenario, objects: Vec<IncomingObject>) -> PullReport {
    pull(
        s,
        PullOutcome {
            objects,
            sync: Some("s1".into()),
            ..PullOutcome::default()
        },
    )
}

fn calls(s: &Scenario) -> Vec<(String, Option<String>, &'static str)> {
    s.fixture
        .hooks
        .pulled
        .borrow()
        .iter()
        .map(|(task, before, pull)| {
            assert_eq!(pull.remote, todoist());
            (task.clone(), before.clone(), pull.change.as_str())
        })
        .collect()
}

fn subject(s: &Scenario, oid: &Oid) -> String {
    s.store.get(oid).unwrap().unwrap().base().subject.clone()
}

fn committed_subject(s: &Scenario, oid: &Oid) -> String {
    s.store
        .committed(oid)
        .unwrap()
        .unwrap()
        .base()
        .subject
        .clone()
}
