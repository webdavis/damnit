use dam_domain::{CommitRecord, Oid};

use crate::config::RemoteConfig;
use crate::errors::{Refusal, UseCaseError};
use crate::ports::{Notice, RemoteHelper, RemoteName};
use crate::remote::RemoteCapabilities;
use crate::service::{Dam, Remotes};
use crate::use_cases::atomically::atomically;
use crate::use_cases::connect::connect;

#[derive(Debug)]
pub struct PushReport {
    pub remote: RemoteName,
    pub sent: usize,
    pub succeeded: usize,
    pub failed: Vec<(Oid, String)>,
    pub skipped: usize,
}

impl Dam<'_> {
    pub fn push(
        &self,
        remotes: Remotes<'_>,
        remote: Option<&str>,
    ) -> Result<Vec<PushReport>, UseCaseError> {
        let conflicts = self.store.conflicts()?.len();
        if conflicts > 0 {
            return Err(Refusal::UnresolvedConflicts(conflicts).into());
        }
        let targets = self.config.select_remotes(remote)?;
        let mut reports = Vec::with_capacity(targets.len());
        for target in targets {
            let (mut helper, caps) = connect(remotes, self.store, target)?;
            reports.push(self.push_one(helper.as_mut(), &caps, target)?);
        }
        Ok(reports)
    }

    fn push_one(
        &self,
        helper: &mut dyn RemoteHelper,
        caps: &RemoteCapabilities,
        remote: &RemoteConfig,
    ) -> Result<PushReport, UseCaseError> {
        let unpushed = self.store.unpushed(&remote.name)?;
        let changes = changes_to_send(self.store, remote, &unpushed)?;
        let Plan { mutations, skipped } = plan(self.store, caps, remote, changes)?;
        let outcomes = if mutations.is_empty() {
            vec![]
        } else {
            helper.push(mutations.clone())?
        };
        let settlement = reconcile(&mutations, outcomes);
        atomically(self.store, || {
            self.settle_push(remote, &unpushed, &settlement)
        })?;
        let sent = mutations.len();
        Ok(PushReport {
            remote: remote.name.clone(),
            sent,
            succeeded: sent.saturating_sub(settlement.failed.len()),
            failed: settlement.failed,
            skipped,
        })
    }

    fn settle_push(
        &self,
        remote: &RemoteConfig,
        unpushed: &[CommitRecord],
        settlement: &Settlement,
    ) -> Result<(), UseCaseError> {
        let store = self.store;
        for (oid, id) in &settlement.map {
            store.map_remote_id(&remote.name, oid, id)?;
        }
        for oid in &settlement.clear {
            store.clear_remote_mapping(&remote.name, oid)?;
        }
        for after in &settlement.snapshot {
            store.set_remote_snapshot(&remote.name, after)?;
        }
        for (oid, why) in &settlement.failed {
            store.add_notice(&Notice::PushFailed {
                remote: remote.name.clone(),
                oid: oid.clone(),
                why: why.clone(),
            })?;
        }
        store.set_push_retries(&remote.name, &settlement.retries())?;
        for id in settlement.pushed(unpushed) {
            store.mark_pushed(&remote.name, id)?;
        }
        store.set_last_push(&remote.name, self.clock.now())?;
        Ok(())
    }
}

mod idempotency;
mod mutations;
mod reconcile;

use mutations::{Plan, changes_to_send, plan};
use reconcile::{Settlement, reconcile};

#[cfg(test)]
mod tests;
