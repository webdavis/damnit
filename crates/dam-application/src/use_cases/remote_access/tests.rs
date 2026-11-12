use std::time::Duration;

use dam_domain::Permission::{RemotePull, RemotePush};
use dam_domain::{Grant, Object, Task};

use crate::config::RemoteConfig;
use crate::errors::{Refusal, UseCaseError};
use crate::ports::{HelperError, RemoteName};
use crate::remote::{IncomingObject, PullOutcome};
use crate::service::Dam;
use crate::testing::prelude::*;
use crate::testing::{Scenario, allowed, answering, config, holder, oid, remotes_launched_by};

fn todoist_and_gcal_both_stale() -> Scenario {
    let mut config = config();
    config.remotes[0].stale = Some(Duration::from_secs(60));
    config.remotes.push(RemoteConfig {
        name: RemoteName("gcal".into()),
        url: "gcal::".into(),
        helper: "gcal".into(),
        ..config.remotes[0].clone()
    });
    Scenario::default().configured(config)
}

fn only(permission: dam_domain::Permission, remote: &str) -> Grant {
    Grant {
        remotes: Some(vec![remote.into()]),
        ..allowed(&[permission], None)
    }
}

fn nothing_upstream() -> Result<PullOutcome, HelperError> {
    Ok(PullOutcome::default())
}

#[test]
fn a_named_remote_the_token_may_not_reach_is_refused() {
    let s = todoist_and_gcal_both_stale();
    let token = holder(vec![only(RemotePull, "todoist")]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    assert_eq!(
        dam.remotes_permitting(RemotePull, Some("gcal")).err(),
        Some(UseCaseError::Refused(Refusal::NotPermitted(RemotePull)))
    );
    assert_eq!(
        dam.remotes_permitting(RemotePull, Some("todoist"))
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn every_remote_narrows_to_the_ones_the_token_may_reach() {
    let s = todoist_and_gcal_both_stale();
    let token = holder(vec![only(RemotePush, "gcal")]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    let names: Vec<&str> = dam
        .remotes_permitting(RemotePush, None)
        .unwrap()
        .iter()
        .map(|r| r.name.0.as_str())
        .collect();
    assert_eq!(names, ["gcal"]);
    assert!(dam.remotes_permitting(RemotePull, None).unwrap().is_empty());
}

#[test]
fn a_tokens_read_never_pulls_a_stale_remote_it_may_not_pull() {
    let s = todoist_and_gcal_both_stale();
    let launcher = answering(nothing_upstream());
    let reader = holder(vec![allowed(&[dam_domain::Permission::TaskRead], None)]);
    let dam = Dam {
        access: &reader,
        ..s.dam()
    };
    dam.refresh_stale(remotes_launched_by(&launcher), |e| format!("{e:?}"))
        .unwrap();
    assert!(launcher.launched_with.borrow().is_empty());
}

#[test]
fn a_tokens_read_pulls_a_stale_remote_it_may_pull() {
    let s = todoist_and_gcal_both_stale();
    let launcher = answering(Ok(PullOutcome {
        objects: vec![IncomingObject {
            remote_id: "r1".into(),
            object: Object::Task(Task::new(oid(0), "upstream")),
        }],
        ..PullOutcome::default()
    }));
    let puller = holder(vec![only(RemotePull, "todoist")]);
    let dam = Dam {
        access: &puller,
        ..s.dam()
    };
    dam.refresh_stale(remotes_launched_by(&launcher), |e| format!("{e:?}"))
        .unwrap();
    assert_eq!(launcher.launched_with.borrow().len(), 1);
    assert_eq!(s.store.all().unwrap().len(), 1);
}

#[test]
fn a_named_remote_outside_the_grant_is_refused_alike_whether_or_not_it_exists() {
    let s = todoist_and_gcal_both_stale();
    let token = holder(vec![only(RemotePull, "todoist")]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    let refused = Some(UseCaseError::Refused(Refusal::NotPermitted(RemotePull)));
    assert_eq!(
        dam.remotes_permitting(RemotePull, Some("gcal")).err(),
        refused
    );
    assert_eq!(
        dam.remotes_permitting(RemotePull, Some("nope")).err(),
        refused
    );
    assert_eq!(
        s.dam().remotes_permitting(RemotePull, Some("nope")).err(),
        Some(UseCaseError::Refused(Refusal::NoSuchRemote("nope".into())))
    );
}
