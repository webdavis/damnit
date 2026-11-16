use dam_domain::Permission;

use crate::errors::{Refusal, UseCaseError};
use crate::service::{Dam, Remotes};
use crate::use_cases::pull::PullReport;
use crate::use_cases::push::PushReport;

#[derive(Debug)]
pub struct SyncReport {
    pub pulled: PullReport,
    pub pushed: PushReport,
}

impl Dam<'_> {
    pub fn sync_now(&self, remotes: Remotes<'_>, remote: &str) -> Result<SyncReport, UseCaseError> {
        self.remotes_permitting(Permission::RemotePush, Some(remote))?;
        let pulled = only(self.pull(remotes, Some(remote))?, remote)?;
        let pushed = only(self.push(remotes, Some(remote))?, remote)?;
        Ok(SyncReport { pulled, pushed })
    }
}

fn only<T>(reports: Vec<T>, remote: &str) -> Result<T, UseCaseError> {
    reports
        .into_iter()
        .next()
        .ok_or_else(|| Refusal::NoSuchRemote(remote.to_string()).into())
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod token_tests;
