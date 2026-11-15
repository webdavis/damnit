use std::time::Duration;

use crate::config::{ConfiguredDuration, RemoteConfig};
use crate::errors::{Refusal, UseCaseError};
use crate::ports::{HelperError, RemoteName};
use crate::service::{Dam, Remotes};
use crate::use_cases::freshness::{age_of_last_pull, is_stale};
use crate::use_cases::pull::PullReport;
use crate::use_cases::push::PushReport;

mod failure_notice;

#[derive(Debug)]
pub struct SyncReport {
    pub remote: RemoteName,
    pub sync_every: ConfiguredDuration,
    pub outcome: SyncOutcome,
}

#[derive(Debug)]
pub enum SyncOutcome {
    Synced {
        pulled: PullReport,
        pushed: Option<PushReport>,
    },
    NotDue,
    Failed {
        why: String,
    },
}

impl Dam<'_> {
    pub fn sync_due_remotes(
        &self,
        remotes: Remotes<'_>,
        say: fn(&UseCaseError) -> String,
    ) -> Result<Vec<SyncReport>, UseCaseError> {
        let mut reports = Vec::new();
        for remote in &self.config.remotes {
            let Some(every) = &remote.sync_every else {
                continue;
            };
            let outcome = match self.is_due(&remote.name, every.value)? {
                true => self.sync(remotes, remote, say)?,
                false => SyncOutcome::NotDue,
            };
            reports.push(SyncReport {
                remote: remote.name.clone(),
                sync_every: every.clone(),
                outcome,
            });
        }
        Ok(reports)
    }

    fn is_due(&self, remote: &RemoteName, every: Duration) -> Result<bool, UseCaseError> {
        Ok(is_stale(
            age_of_last_pull(self.store, self.clock, remote)?,
            every,
        ))
    }

    fn sync(
        &self,
        remotes: Remotes<'_>,
        remote: &RemoteConfig,
        say: fn(&UseCaseError) -> String,
    ) -> Result<SyncOutcome, UseCaseError> {
        match self.pull_then_push(remotes, remote) {
            Ok((pulled, pushed)) => {
                self.clear_sync_failure(&remote.name)?;
                Ok(SyncOutcome::Synced { pulled, pushed })
            }
            Err(e) if cancelled(&e) => Err(e),
            Err(e) => {
                let why = say(&e.withholding_what_credential_code_raised());
                self.replace_sync_failure(&remote.name, &why)?;
                Ok(SyncOutcome::Failed { why })
            }
        }
    }

    fn pull_then_push(
        &self,
        remotes: Remotes<'_>,
        remote: &RemoteConfig,
    ) -> Result<(PullReport, Option<PushReport>), UseCaseError> {
        let name = Some(remote.name.0.as_str());
        let pulled = only(self.pull(remotes, name)?, remote)?;
        if self.takes_no_kinds(&remote.name)? {
            return Ok((pulled, None));
        }
        let pushed = only(self.push(remotes, name)?, remote)?;
        Ok((pulled, Some(pushed)))
    }

    fn takes_no_kinds(&self, remote: &RemoteName) -> Result<bool, UseCaseError> {
        let kinds = self.store.accepted_kinds(remote)?;
        Ok(kinds.is_some_and(|kinds| kinds.is_empty()))
    }
}

fn only<T>(reports: Vec<T>, remote: &RemoteConfig) -> Result<T, UseCaseError> {
    reports
        .into_iter()
        .next()
        .ok_or_else(|| Refusal::NoSuchRemote(remote.name.0.clone()).into())
}

fn cancelled(e: &UseCaseError) -> bool {
    matches!(
        e,
        UseCaseError::Helper(HelperError::Cancelled)
            | UseCaseError::PushStopped {
                by: HelperError::Cancelled,
                ..
            }
    )
}

#[cfg(test)]
mod failure_tests;
#[cfg(test)]
mod tests;
