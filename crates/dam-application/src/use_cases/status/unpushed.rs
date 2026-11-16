use std::collections::BTreeSet;

use dam_domain::{Change, Kind};

use super::{Unpushed, taken_by};
use crate::errors::UseCaseError;
use crate::ports::RemoteName;
use crate::service::Dam;

impl Dam<'_> {
    pub(super) fn unpushed_for(
        &self,
        remote: &RemoteName,
        accepted: Option<&[Kind]>,
    ) -> Result<(Unpushed, bool), UseCaseError> {
        let mut commit_ids = Vec::new();
        let mut oids = Vec::new();
        let mut named = BTreeSet::new();
        let mut hidden = false;
        let newest_first = self.store.unpushed(remote)?.into_iter().rev();
        for record in newest_first {
            let mut owed: Vec<&Change> = Vec::new();
            for change in record.changes.iter().map(|authored| &authored.change) {
                if !taken_by(accepted, change) {
                    continue;
                }
                match self.access.token().is_none() || self.reads(change.oid())? {
                    true => owed.push(change),
                    false => hidden = true,
                }
            }
            if owed.is_empty() {
                continue;
            }
            for change in owed {
                if named.insert(change.oid().clone()) {
                    oids.push(change.oid().clone());
                }
            }
            commit_ids.push(record.id);
        }
        let unpushed = Unpushed {
            remote: remote.clone(),
            commit_ids,
            oids,
        };
        Ok((unpushed, hidden))
    }
}
