use dam_domain::{CommitRecord, Oid, Permission, Subject};

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
            let shown = self
                .access
                .permits(Permission::ConflictRead, &[Subject::Whole])
                .then_some(conflicts);
            return Err(Refusal::UnresolvedConflicts(shown).into());
        }
        let targets = self.remotes_permitting(Permission::RemotePush, remote)?;
        let mut reports = Vec::with_capacity(targets.len());
        for target in targets {
            let (mut helper, caps) = connect(remotes, self.store, self.access, target)?;
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
        let doubt = self.store.fields_in_doubt(&remote.name)?;
        let changes = changes_to_send(self.store, remote, &unpushed, &doubt)?;
        let removals = Removals::new(self.store, &remote.name, caps, &changes)?;
        let refiles = Refiles::new(self.store, &remote.name, caps, &changes)?;
        let Plan { mutations, skipped } =
            plan(self.store, caps, remote, changes, &refiles, &doubt)?;
        let sent = send_in_waves(helper, removals, mutations)?;
        atomically(self.store, || {
            self.settle_push(remote, &unpushed, &sent.settlement)
        })?;
        if let Some(by) = sent.stopped_by {
            return Err(UseCaseError::PushStopped {
                by,
                kept: sent.answered,
                unanswered: sent.settlement.unanswered.len(),
            });
        }
        let refused = sent.settlement.failed.len() - sent.held;
        Ok(PushReport {
            remote: remote.name.clone(),
            sent: sent.mutations,
            succeeded: sent.mutations.saturating_sub(refused),
            failed: sent.settlement.failed,
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
        for oid in &settlement.answered {
            store.count_answer(&remote.name, oid)?;
        }
        settle_doubt(store, &remote.name, settlement)?;
        store.set_push_retries(&remote.name, &settlement.retries())?;
        for id in settlement.pushed(unpushed) {
            store.mark_pushed(&remote.name, id)?;
        }
        store.set_last_push(&remote.name, self.clock.now())?;
        Ok(())
    }
}

mod doubt;
mod idempotency;
mod mutations;
mod owed;
mod reconcile;
mod refiles;
mod removals;
mod sending_order;
mod waves;

use doubt::settle_doubt;
use mutations::{Plan, plan};
use owed::changes_to_send;
use reconcile::Settlement;
use refiles::Refiles;
use removals::Removals;
use waves::send_in_waves;

#[cfg(test)]
mod tests;
