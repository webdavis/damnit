use dam_domain::{CommitRecord, Oid, Permission, Subject};

use crate::config::RemoteConfig;
use crate::errors::{Refusal, UseCaseError};
use crate::ports::{RemoteHelper, RemoteName};
use crate::remote::{RemoteCapabilities, RemoteMutation};
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
        let record = self.store.attempts(&remote.name)?;
        let changes = changes_to_send(self.store, remote, &unpushed, &record)?;
        let mut sent = self.push_round(helper, caps, remote, &unpushed, changes, &[])?;
        if let (true, Some(by)) = (sent.stopped_before_any_answer, &sent.stopped_by) {
            return Err(by.clone().into());
        }
        let follow_ups = follow_ups(self.store, &remote.name, &sent.settlement.created)?;
        if sent.stopped_by.is_none() && !follow_ups.is_empty() {
            let retries = sent.settlement.retries();
            let later = self.push_round(helper, caps, remote, &[], follow_ups, &retries)?;
            sent.absorb(later);
        }
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
            skipped: sent.skipped,
        })
    }

    fn push_round(
        &self,
        helper: &mut dyn RemoteHelper,
        caps: &RemoteCapabilities,
        remote: &RemoteConfig,
        unpushed: &[CommitRecord],
        changes: OwedChanges,
        earlier_retries: &[Oid],
    ) -> Result<Sent, UseCaseError> {
        let record = self.store.attempts(&remote.name)?;
        let dropped = creates_no_longer_sent(self.store, remote, caps, unpushed, &record)?;
        let removals = Removals::new(self.store, &remote.name, caps, &changes)?;
        let refiles = Refiles::new(self.store, &remote.name, caps, &changes)?;
        let Plan {
            mut mutations,
            skipped,
            unused,
        } = plan(self.store, helper, caps, remote, changes, &refiles, &record)?;
        let attempting = attempting(self.store, helper, &remote.name, &record, &mut mutations)?;
        let record = |ready: &[RemoteMutation]| {
            record_before_the_call(self.store, &remote.name, ready, &attempting)
        };
        let mut sent = send_in_waves(helper, removals, mutations, &record)?;
        sent.skipped = skipped;
        sent.settlement.dropped = dropped;
        sent.settlement.dropped.unused.extend(unused);
        if !sent.stopped_before_any_answer {
            atomically(self.store, || {
                self.settle_push(
                    remote,
                    unpushed,
                    &sent.settlement,
                    &attempting,
                    earlier_retries,
                )
            })?;
        }
        Ok(sent)
    }
}

mod attempting;
mod attempts;
mod dropped;
mod idempotency;
mod mutations;
mod owed;
mod reconcile;
mod refiles;
mod removals;
mod sending_order;
mod settle;
mod waves;

use attempting::attempting;
use attempts::record_before_the_call;
use dropped::creates_no_longer_sent;
use mutations::{Plan, plan};
use owed::{OwedChanges, changes_to_send, follow_ups};
use refiles::Refiles;
use removals::Removals;
use waves::{Sent, send_in_waves};

#[cfg(test)]
mod tests;
