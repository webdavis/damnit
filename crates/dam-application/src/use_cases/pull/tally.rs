use dam_domain::Oid;

use super::PullReport;
use crate::errors::UseCaseError;
use crate::ports::RemoteName;
use crate::service::Dam;

#[derive(Default)]
pub(super) struct Tally {
    pub(super) created: Vec<Oid>,
    pub(super) updated: Vec<Oid>,
    pub(super) conflicts: Vec<Oid>,
    pub(super) removed_upstream: Vec<Oid>,
    pub(super) unchanged: Vec<Oid>,
    pub(super) creates_matched: Vec<Oid>,
    pub(super) values_kept: Vec<(Oid, usize)>,
    pub(super) deletes_confirmed: Vec<Oid>,
}

impl Dam<'_> {
    pub(super) fn counted(
        &self,
        remote: &RemoteName,
        tally: Tally,
    ) -> Result<PullReport, UseCaseError> {
        let owners = self.access.token().is_none();
        let mut unseen_pulled = false;
        let mut count = |oids: &[Oid]| -> Result<usize, UseCaseError> {
            let mut read = 0;
            for oid in oids {
                match owners || self.reads(oid)? {
                    true => read += 1,
                    false => unseen_pulled = true,
                }
            }
            Ok(read)
        };
        let created = count(&tally.created)?;
        let updated = count(&tally.updated)?;
        let conflicts = count(&tally.conflicts)?;
        let removed_upstream = count(&tally.removed_upstream)?;
        let unchanged = count(&tally.unchanged)?;
        let creates_matched = count(&tally.creates_matched)?;
        let deletes_confirmed = count(&tally.deletes_confirmed)?;
        let mut values_kept = 0;
        for (oid, kept) in &tally.values_kept {
            match owners || self.reads(oid)? {
                true => values_kept += kept,
                false => unseen_pulled = true,
            }
        }
        Ok(PullReport {
            remote: remote.clone(),
            created,
            updated,
            conflicts,
            removed_upstream,
            unchanged,
            creates_matched,
            values_kept,
            deletes_confirmed,
            task_pulled: None,
            unseen_pulled,
        })
    }
}
