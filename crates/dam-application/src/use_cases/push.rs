use dam_domain::{CommitRecord, Oid, Permission, Subject};

use crate::config::RemoteConfig;
use crate::errors::UseCaseError;
use crate::ports::{RemoteHelper, RemoteName};
use crate::remote::{RemoteCapabilities, RemoteMutation};
use crate::service::{Dam, Remotes};
use crate::use_cases::atomically::atomically;

#[derive(Debug)]
pub struct PushReport {
    pub remote: RemoteName,
    pub sent: usize,
    pub succeeded: usize,
    pub failed: Vec<(Oid, String)>,
    pub skipped: usize,
    pub held: Option<usize>,
}

impl Dam<'_> {
    pub fn push(
        &self,
        remotes: Remotes<'_>,
        remote: Option<&str>,
    ) -> Result<Vec<PushReport>, UseCaseError> {
        let sees_conflicts = self
            .access
            .permits(Permission::ConflictRead, &[Subject::Whole]);
        let targets = self.remotes_permitting(Permission::RemotePush, remote)?;
        let mut reports = Vec::with_capacity(targets.len());
        for target in targets {
            let (mut helper, caps) = self.connect(remotes, target)?;
            let mut report = self.push_one(helper.as_mut(), &caps, target)?;
            report.held = report.held.filter(|n| *n == 0 || sees_conflicts);
            reports.push(report);
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
        let owed = hold_for_conflicts(self.store, changes)?;
        let held = owed.held.len();
        let mut sent = self.push_round(helper, caps, remote, &unpushed, owed, &[])?;
        if let (true, Some(by)) = (sent.stopped_before_any_answer, &sent.stopped_by) {
            return Err(by.clone().into());
        }
        let follow_ups = follow_ups(self.store, &remote.name, &sent.settlement.created)?;
        if sent.stopped_by.is_none() && !follow_ups.is_empty() {
            let retries = sent.settlement.retries();
            let later = self.push_round(helper, caps, remote, &[], follow_ups.into(), &retries)?;
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
            held: Some(held),
        })
    }

    fn push_round(
        &self,
        helper: &mut dyn RemoteHelper,
        caps: &RemoteCapabilities,
        remote: &RemoteConfig,
        unpushed: &[CommitRecord],
        owed: Owed,
        earlier_retries: &[Oid],
    ) -> Result<Sent, UseCaseError> {
        let Owed { changes, held } = owed;
        let record = self.store.attempts(&remote.name)?;
        let dropped = creates_no_longer_sent(self.store, remote, caps, unpushed, &record)?;
        let removals = Removals::new(self.store, &remote.name, &changes)?;
        let Plan {
            mut mutations,
            skipped,
            unused,
            unplaced,
        } = plan(self.store, helper, caps, remote, changes, &record)?;
        let attempting = attempting(self.store, helper, &remote.name, &record, &mut mutations)?;
        let record = |ready: &[RemoteMutation]| {
            record_before_the_call(self.store, &remote.name, ready, &attempting)
        };
        let mut sent = self.sending(helper, &remote.name, removals, mutations, &record)?;
        sent.skipped = skipped;
        sent.held += unplaced.len();
        sent.settlement.failed.extend(unplaced);
        sent.settlement.held = held;
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
mod conflict_holds;
mod containers;
mod dropped;
mod failure_notices;
mod idempotency;
mod mutations;
mod owed;
mod reconcile;
mod removals;
mod sending;
mod sending_order;
mod settle;
mod waves;

use attempting::attempting;
use attempts::record_before_the_call;
use conflict_holds::{Owed, hold_for_conflicts};
use dropped::creates_no_longer_sent;
use mutations::{Plan, plan};
use owed::{changes_to_send, follow_ups};
use removals::Removals;
use waves::Sent;

#[cfg(test)]
mod tests;
