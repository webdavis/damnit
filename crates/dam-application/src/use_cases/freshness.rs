//! A reader's bound on how long ago a remote last pulled.

use std::time::Duration;

use crate::config::Config;
use crate::errors::{Refusal, UseCaseError};
use crate::ports::{Clock, RemoteTrackingRepository};

/// Refuses when a named remote last pulled successfully longer ago than its
/// bound, or never has. A clock set back since the pull reads as age zero.
pub fn require_fresh(
    remote_tracking: &dyn RemoteTrackingRepository,
    clock: &dyn Clock,
    config: &Config,
    bounds: &[(String, Duration)],
) -> Result<(), UseCaseError> {
    let now = clock.now();
    for (name, bound) in bounds {
        let remote = config
            .remote(name)
            .ok_or_else(|| Refusal::NoSuchRemote(name.clone()))?;
        let age = remote_tracking
            .last_pull(&remote.name)?
            .map(|at| u64::try_from(now.duration_since(at).as_secs()).unwrap_or(0));
        let limit = bound.as_secs();
        if age.is_none_or(|a| a > limit) {
            return Err(Refusal::StaleRemote {
                remote: name.clone(),
                age,
                limit,
            }
            .into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use jiff::SignedDuration;
    use jiff::civil::date;

    use super::*;
    use crate::config::RemoteConfig;
    use crate::errors::Refusal;
    use crate::ports::RemoteName;
    use crate::testing::prelude::*;
    use crate::testing::{FixedClock, MemoryStore};

    const HOUR: Duration = Duration::from_secs(3600);

    fn clock() -> FixedClock {
        FixedClock(date(2026, 9, 18))
    }

    fn config() -> Config {
        Config {
            remotes: vec![RemoteConfig {
                name: RemoteName("gcal".into()),
                helper: "gcal".into(),
                url: "gcal::".into(),
                credentials: vec![],
                stale: None,
                deadline: None,
                path: None,
            }],
            ..Config::default()
        }
    }

    fn pulled_ago(ago: SignedDuration) -> MemoryStore {
        let store = MemoryStore::new();
        let at = clock().now().checked_sub(ago).unwrap();
        store.set_last_pull(&RemoteName("gcal".into()), at).unwrap();
        store
    }

    fn check(store: &MemoryStore, remote: &str) -> Result<(), UseCaseError> {
        require_fresh(store, &clock(), &config(), &[(remote.into(), HOUR)])
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
        assert_eq!(
            refused.to_string(),
            "remote \"gcal\" last pulled 7200s ago, longer than --max-age allows (3600s); run dam pull gcal"
        );
    }

    #[test]
    fn a_pull_exactly_as_old_as_the_bound_answers() {
        assert_eq!(
            check(&pulled_ago(SignedDuration::from_hours(1)), "gcal"),
            Ok(())
        );
    }

    /// A pull stamped after now, as when the clock was set back, is fresh.
    #[test]
    fn a_pull_from_the_future_reads_as_age_zero() {
        assert_eq!(
            check(&pulled_ago(SignedDuration::from_hours(-1)), "gcal"),
            Ok(())
        );
    }

    #[test]
    fn every_bound_is_held_not_only_the_first() {
        let store = pulled_ago(SignedDuration::from_hours(2));
        let bounds = [("gcal".to_string(), 3 * HOUR), ("gcal".to_string(), HOUR)];
        let refused = require_fresh(&store, &clock(), &config(), &bounds).unwrap_err();
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
        let refused = check(&MemoryStore::new(), "gcal").unwrap_err();
        assert_eq!(
            refused.to_string(),
            "remote \"gcal\" has never been pulled; run dam pull gcal"
        );
    }

    #[test]
    fn a_bound_on_a_remote_the_config_does_not_name_is_no_such_remote() {
        let refused = check(&MemoryStore::new(), "nope").unwrap_err();
        assert_eq!(
            refused,
            UseCaseError::Refused(Refusal::NoSuchRemote("nope".into()))
        );
    }
}
