use std::time::Duration;

use dam_domain::Permission;

use crate::errors::{Refusal, UseCaseError};
use crate::ports::RemoteName;
use crate::service::{Dam, Remotes};
use crate::use_cases::sync_now::SyncReport;

impl Dam<'_> {
    pub fn sync_when_due(
        &self,
        remotes: Remotes<'_>,
        remote: &str,
        interval: Option<Duration>,
    ) -> Result<Option<SyncReport>, UseCaseError> {
        let configured = self
            .remotes_permitting(Permission::RemotePush, Some(remote))?
            .into_iter()
            .next()
            .ok_or_else(|| Refusal::NoSuchRemote(remote.to_string()))?;
        let interval = interval.unwrap_or_else(|| configured.sync.interval());
        if !configured.sync.allowed() || !self.due(&configured.name, interval)? {
            return Ok(None);
        }
        self.sync_now(remotes, remote).map(Some)
    }

    fn due(&self, remote: &RemoteName, interval: Duration) -> Result<bool, UseCaseError> {
        let Some(at) = self.store.last_pull(remote)? else {
            return Ok(true);
        };
        let since = self.clock.now().duration_since(at).as_secs();
        Ok(u64::try_from(since).unwrap_or(0) >= interval.as_secs())
    }
}

#[cfg(test)]
mod tests;
