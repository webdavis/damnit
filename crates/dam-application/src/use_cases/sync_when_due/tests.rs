use std::time::Duration;

use jiff::SignedDuration;

use super::*;
use crate::config::RemoteSync;
use crate::testing::prelude::*;
use crate::testing::{Scenario, accepting_all, config, pushing, remotes_launched_by, todoist};

fn when_due(s: &Scenario, interval: Option<Duration>) -> Option<SyncReport> {
    let (launcher, _) = pushing(accepting_all(None));
    s.dam()
        .sync_when_due(remotes_launched_by(&launcher), "todoist", interval)
        .unwrap()
}

fn pulled_minutes_ago(minutes: i64) -> Scenario {
    let s = Scenario::default();
    s.pulled_ago(SignedDuration::from_mins(minutes));
    s
}

#[test]
fn a_remote_never_synced_is_due() {
    assert!(when_due(&Scenario::default(), None).is_some());
}

#[test]
fn a_remote_synced_longer_ago_than_its_interval_is_synced() {
    let s = pulled_minutes_ago(16);
    assert!(when_due(&s, None).is_some());
}

#[test]
fn a_remote_synced_inside_its_interval_is_left_alone() {
    let s = pulled_minutes_ago(5);
    let before = s.store.last_pull(&todoist()).unwrap();
    assert!(when_due(&s, None).is_none());
    assert_eq!(s.store.last_pull(&todoist()).unwrap(), before);
}

#[test]
fn a_given_interval_replaces_the_configured_one() {
    let s = pulled_minutes_ago(5);
    assert!(when_due(&s, Some(Duration::from_secs(3 * 60))).is_some());
}

#[test]
fn a_remote_whose_sync_is_not_allowed_is_never_due() {
    let mut config = config();
    config.remotes[0].sync = RemoteSync {
        allowed: Some(false),
        ..RemoteSync::default()
    };
    let s = Scenario::default().configured(config);
    assert!(when_due(&s, None).is_none());
    assert!(s.store.last_pull(&todoist()).unwrap().is_none());
}

#[test]
fn an_unknown_remote_is_refused_as_a_pull_refuses_it() {
    let s = Scenario::default();
    let (launcher, _) = pushing(accepting_all(None));
    let refused = s
        .dam()
        .sync_when_due(remotes_launched_by(&launcher), "nope", None)
        .unwrap_err();
    let pulled = s
        .dam()
        .pull(remotes_launched_by(&launcher), Some("nope"))
        .unwrap_err();
    assert_eq!(format!("{refused:?}"), format!("{pulled:?}"));
}

#[test]
fn a_token_without_the_push_permission_is_refused_before_anything_is_read() {
    let mut config = config();
    config.remotes[0].sync.allowed = Some(false);
    let s = Scenario::default().configured(config);
    let token = crate::testing::holder(vec![crate::testing::allowed(
        &[dam_domain::Permission::RemotePull],
        None,
    )]);
    let dam = crate::service::Dam {
        access: &token,
        ..s.dam()
    };
    let (launcher, _) = pushing(accepting_all(None));
    let refused = dam
        .sync_when_due(remotes_launched_by(&launcher), "todoist", None)
        .unwrap_err();
    assert!(matches!(
        refused,
        UseCaseError::Refused(crate::errors::Refusal::NotPermitted(
            dam_domain::Permission::RemotePush
        ))
    ));
}
