use std::cell::RefCell;
use std::rc::Rc;

use super::super::remote;
use crate::config::{Config, CredentialSpec, RemoteConfig};
use crate::remote::{
    IncomingObject, MutationOutcome, MutationVerdict, PullOutcome, RemoteMutation,
};
use crate::testing::{ScriptedHelper, ScriptedLauncher, oid, task_caps};
use dam_domain::{Object, Task};

pub(super) fn config() -> Config {
    Config {
        remotes: vec![RemoteConfig {
            name: remote(),
            helper: "todoist".into(),
            url: "todoist::".into(),
            credentials: vec![CredentialSpec::Literal {
                name: "api_token".into(),
                value: "t".into(),
            }],
            stale: None,
            deadline: None,
            path: None,
        }],
        ..Config::default()
    }
}

pub(super) fn upstream(subject: &str) -> IncomingObject {
    IncomingObject {
        remote_id: "r1".into(),
        object: Object::Task(Task::new(oid(0), subject)),
    }
}

pub(super) fn launcher_that_accepts_every_push(objects: Vec<IncomingObject>) -> ScriptedLauncher {
    recording_launcher_that_accepts_every_push(objects).0
}

pub(super) fn recording_launcher_that_accepts_every_push(
    objects: Vec<IncomingObject>,
) -> (ScriptedLauncher, Rc<RefCell<Vec<RemoteMutation>>>) {
    let pushed = Rc::new(RefCell::new(Vec::new()));
    let seen = pushed.clone();
    let l = ScriptedLauncher {
        make: Box::new(move || ScriptedHelper {
            caps: task_caps(),
            pull_answer: Ok(PullOutcome {
                objects: objects.clone(),
                ..PullOutcome::default()
            }),
            push_answer: Box::new(|ms| {
                Ok(ms
                    .iter()
                    .map(|m| MutationOutcome {
                        oid: m.oid.clone(),
                        verdict: MutationVerdict::Accepted {
                            remote_id: Some("r1".into()),
                        },
                    })
                    .collect())
            }),
            pushed: seen.clone(),
            pulled_since: Rc::new(RefCell::new(vec![])),
        }),
        launched_with: Rc::new(RefCell::new(vec![])),
    };
    (l, pushed)
}
