use dam_domain::Grant;
use dam_domain::Permission::{RemotePull, RemotePush};

use crate::config::RemoteConfig;
use crate::errors::{Refusal, UseCaseError};
use crate::ports::RemoteName;
use crate::service::Dam;
use crate::testing::{Scenario, allowed, config, holder};

fn todoist_and_gcal() -> Scenario {
    let mut config = config();
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

#[test]
fn a_named_remote_the_token_may_not_reach_is_refused() {
    let s = todoist_and_gcal();
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
    let s = todoist_and_gcal();
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
fn a_named_remote_outside_the_grant_is_refused_alike_whether_or_not_it_exists() {
    let s = todoist_and_gcal();
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
