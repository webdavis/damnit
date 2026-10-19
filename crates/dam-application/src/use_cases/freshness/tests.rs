use std::time::Duration;

use jiff::SignedDuration;

use super::*;
use crate::errors::Refusal;
use crate::ports::RemoteName;
use crate::testing::{Scenario, config};

const HOUR: Duration = Duration::from_secs(3600);

fn gcal() -> Scenario {
    let mut config = config();
    config.remotes[0].name = RemoteName("gcal".into());
    Scenario::default().configured(config)
}

fn pulled_ago(ago: SignedDuration) -> Scenario {
    let s = gcal();
    s.pulled_ago(ago);
    s
}

fn check(s: &Scenario, remote: &str) -> Result<(), UseCaseError> {
    s.dam().require_fresh(&[(remote.into(), HOUR)])
}

#[test]
fn a_pull_inside_the_bound_answers() {
    assert_eq!(
        check(&pulled_ago(SignedDuration::from_mins(59)), "gcal"),
        Ok(())
    );
}

#[test]
fn a_pull_older_than_the_bound_is_refused_with_its_age() {
    let refused = check(&pulled_ago(SignedDuration::from_hours(2)), "gcal").unwrap_err();
    assert_eq!(
        refused,
        UseCaseError::Refused(Refusal::StaleRemote {
            remote: "gcal".into(),
            age: Some(7200),
            limit: 3600
        })
    );
}

#[test]
fn a_pull_exactly_as_old_as_the_bound_answers() {
    assert_eq!(
        check(&pulled_ago(SignedDuration::from_hours(1)), "gcal"),
        Ok(())
    );
}

#[test]
fn a_pull_from_the_future_reads_as_age_zero() {
    assert_eq!(
        check(&pulled_ago(SignedDuration::from_hours(-1)), "gcal"),
        Ok(())
    );
}

#[test]
fn every_bound_is_held_not_only_the_first() {
    let s = pulled_ago(SignedDuration::from_hours(2));
    let bounds = [("gcal".to_string(), 3 * HOUR), ("gcal".to_string(), HOUR)];
    let refused = s.dam().require_fresh(&bounds).unwrap_err();
    assert_eq!(
        refused,
        UseCaseError::Refused(Refusal::StaleRemote {
            remote: "gcal".into(),
            age: Some(7200),
            limit: 3600
        })
    );
}

#[test]
fn a_remote_never_pulled_is_refused() {
    let refused = check(&gcal(), "gcal").unwrap_err();
    assert_eq!(
        refused,
        UseCaseError::Refused(Refusal::StaleRemote {
            remote: "gcal".into(),
            age: None,
            limit: 3600
        })
    );
}

#[test]
fn a_bound_on_a_remote_the_config_does_not_name_is_no_such_remote() {
    let refused = check(&gcal(), "nope").unwrap_err();
    assert_eq!(
        refused,
        UseCaseError::Refused(Refusal::NoSuchRemote("nope".into()))
    );
}
