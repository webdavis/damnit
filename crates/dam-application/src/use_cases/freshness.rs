use std::time::Duration;

use dam_domain::Permission;

use crate::errors::{Refusal, UseCaseError};
use crate::ports::{Clock, RemoteName, RemoteTrackingRepository, StoreError};
use crate::service::Dam;

impl Dam<'_> {
    pub fn require_fresh(&self, bounds: &[(String, Duration)]) -> Result<(), UseCaseError> {
        for (name, bound) in bounds {
            let remote = self
                .config
                .remote(name)
                .filter(|r| self.may_reach(Permission::RemoteRead, r))
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

pub(crate) fn is_stale(age: Option<Duration>, window: Duration) -> bool {
    age.is_none_or(|a| a.as_secs() > window.as_secs())
}

pub(crate) fn age_of_last_pull(
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
mod access_tests;
#[cfg(test)]
mod tests;
