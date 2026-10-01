use std::time::Duration;

use crate::config::Config;
use crate::errors::{Refusal, UseCaseError};
use crate::ports::{Clock, RemoteTrackingRepository};

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
mod tests;
