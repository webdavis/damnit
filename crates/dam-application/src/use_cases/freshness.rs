use std::time::Duration;

use crate::config::Config;
use crate::errors::{Refusal, UseCaseError};
use crate::ports::{
    Clock, CredentialSource, HelperLauncher, Notice, Randomness, RemoteName,
    RemoteTrackingRepository, Repositories, StoreError,
};
use crate::service::Dam;
use crate::use_cases::pull::pull;

impl Dam<'_> {
    pub fn require_fresh(&self, bounds: &[(String, Duration)]) -> Result<(), UseCaseError> {
        for (name, bound) in bounds {
            let remote = self
                .config
                .remote(name)
                .ok_or_else(|| Refusal::NoSuchRemote(name.clone()))?;
            let age = age_of_last_pull(self.store, self.clock, &remote.name)?;
            if is_stale(age, *bound) {
                return Err(Refusal::StaleRemote {
                    remote: name.clone(),
                    age: age.map(|a| a.as_secs()),
                    limit: bound.as_secs(),
                }
                .into());
            }
        }
        Ok(())
    }
}

pub fn refresh_stale(
    repos: Repositories<'_>,
    launcher: &dyn HelperLauncher,
    credentials: &dyn CredentialSource,
    clock: &dyn Clock,
    random: &dyn Randomness,
    config: &Config,
) -> Result<(), UseCaseError> {
    for remote in &config.remotes {
        let Some(window) = remote.stale else {
            continue;
        };
        let age = age_of_last_pull(repos.remote_tracking, clock, &remote.name)?;
        if is_stale(age, window) {
            let pulled = pull(
                repos,
                launcher,
                credentials,
                clock,
                random,
                config,
                Some(&remote.name.0),
            );
            if let Err(e) = pulled {
                repos.notices.add_notice(&Notice::PullFailed {
                    remote: remote.name.clone(),
                    why: e.to_string(),
                })?;
            }
        }
    }
    Ok(())
}

fn is_stale(age: Option<Duration>, window: Duration) -> bool {
    age.is_none_or(|a| a.as_secs() > window.as_secs())
}

fn age_of_last_pull(
    tracking: &dyn RemoteTrackingRepository,
    clock: &dyn Clock,
    remote: &RemoteName,
) -> Result<Option<Duration>, StoreError> {
    let now = clock.now();
    Ok(tracking.last_pull(remote)?.map(|at| {
        Duration::from_secs(u64::try_from(now.duration_since(at).as_secs()).unwrap_or(0))
    }))
}

#[cfg(test)]
mod refresh_tests;
#[cfg(test)]
mod tests;
