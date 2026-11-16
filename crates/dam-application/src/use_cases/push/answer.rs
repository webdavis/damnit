use dam_domain::{Oid, Permission, Subject};

use super::reconcile::Heard;
use super::report::PushReport;
use super::waves::Sent;
use crate::errors::UseCaseError;
use crate::ports::{HelperError, RemoteName};
use crate::service::Dam;
use crate::use_cases::sight::HELD_BY_WHAT_YOU_CANNOT_SEE;

const REFUSED_FOR_A_REASON_THE_OWNER_SEES: &str =
    "refused by the remote; ask the owner to run dam status for the reason";

impl Dam<'_> {
    pub(super) fn answer(
        &self,
        remote: &RemoteName,
        sent: Sent,
    ) -> Result<PushReport, UseCaseError> {
        if self.access.token().is_some() {
            return self.as_seen_answer(remote, sent);
        }
        let refused = sent.settlement.failed.len() - sent.holds.len();
        Ok(PushReport {
            remote: remote.clone(),
            sent: sent.mutations,
            succeeded: sent.mutations.saturating_sub(refused),
            failed: sent.settlement.failed,
            skipped: sent.skipped.len(),
            held: Some(sent.settlement.held.len()),
            unseen_failed: false,
        })
    }

    pub(super) fn stopped(
        &self,
        by: HelperError,
        sent: &Sent,
    ) -> Result<UseCaseError, UseCaseError> {
        let unanswered = &sent.settlement.unanswered;
        let (kept, unanswered) = match self.access.token() {
            None => (sent.answered, unanswered.len()),
            Some(_) => {
                let answered = sent.settlement.heard.iter().map(|(oid, _)| oid);
                (
                    self.count_read(answered)?,
                    self.count_read(unanswered.iter())?,
                )
            }
        };
        Ok(UseCaseError::PushStopped {
            by,
            kept,
            unanswered,
        })
    }

    fn as_seen_answer(&self, remote: &RemoteName, sent: Sent) -> Result<PushReport, UseCaseError> {
        let settlement = &sent.settlement;
        let mut failed = Vec::new();
        for (oid, why) in &settlement.failed {
            if self.reads(oid)? {
                failed.push((oid.clone(), self.as_seen_reason(&sent, oid, why)?));
            }
        }
        let held = self.count_read(settlement.held.iter())?;
        let sees_conflicts = self
            .access
            .permits(Permission::ConflictRead, &[Subject::Whole]);
        let unseen_failed = failed.len() < settlement.failed.len() || held < settlement.held.len();
        let answered = settlement.heard.iter().map(|(oid, _)| oid);
        let accepted = settlement
            .heard
            .iter()
            .filter(|(_, h)| *h == Heard::Accepted);
        Ok(PushReport {
            remote: remote.clone(),
            sent: self.count_read(answered.chain(&settlement.unanswered))?,
            succeeded: self.count_read(accepted.map(|(oid, _)| oid))?,
            skipped: self.count_read(sent.skipped.iter())?,
            held: match (sees_conflicts, settlement.held.is_empty()) {
                (true, _) => Some(held),
                (false, true) => Some(0),
                (false, false) => None,
            },
            unseen_failed: unseen_failed || (!sees_conflicts && !settlement.held.is_empty()),
            failed,
        })
    }

    fn as_seen_reason(&self, sent: &Sent, oid: &Oid, why: &str) -> Result<String, UseCaseError> {
        let hidden = match sent.holds.iter().find(|hold| &hold.oid == oid) {
            Some(hold) if self.any_hidden(&hold.waits_on)? => Some(HELD_BY_WHAT_YOU_CANNOT_SEE),
            Some(_) => None,
            None if !self.reads_everything() => Some(REFUSED_FOR_A_REASON_THE_OWNER_SEES),
            None => None,
        };
        Ok(hidden.unwrap_or(why).to_string())
    }

    fn count_read<'a>(&self, oids: impl Iterator<Item = &'a Oid>) -> Result<usize, UseCaseError> {
        let mut read = 0;
        for oid in oids {
            read += usize::from(self.reads(oid)?);
        }
        Ok(read)
    }
}
