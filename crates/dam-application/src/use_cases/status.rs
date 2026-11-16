use std::collections::BTreeMap;

use dam_domain::{Authored, Change, CommitId, Item, ItemHash, Kind, Object, Oid, Permission};

use crate::errors::UseCaseError;
use crate::ports::{Conflict, Notice, RemoteName, Store};
use crate::service::Dam;
use crate::use_cases::remove::RemovePlan;
use crate::use_cases::stage::working_and_ever_committed_oids;

pub struct Status {
    pub staged: Vec<Authored>,
    pub unstaged: Vec<Change>,
    pub waiting: Vec<Item>,
    pub removal_plans: BTreeMap<ItemHash, RemovePlan>,
    pub conflicts: Vec<Conflict>,
    pub notices: Vec<Notice>,
    pub unpushed: Vec<Unpushed>,
    pub unseen_failed: bool,
}

pub struct Unpushed {
    pub remote: RemoteName,
    pub commit_ids: Vec<CommitId>,
    pub oids: Vec<Oid>,
}

impl Dam<'_> {
    pub fn status(&self, remotes: &[RemoteName]) -> Result<Status, UseCaseError> {
        let mut unpushed = Vec::new();
        let mut unseen_failed = false;
        for remote in remotes {
            let accepted = self.store.accepted_kinds(remote)?;
            let (owed, hidden) = self.unpushed_for(remote, accepted.as_deref())?;
            unseen_failed |= hidden;
            unpushed.push(owed);
        }
        let waiting = self.waiting_items()?;
        let (conflicts, hidden_conflicts) = self.seen_conflicts()?;
        let (notices, hidden_notices) = self.seen_notices()?;
        Ok(Status {
            staged: self.seen_staged()?,
            unstaged: self.diff_working()?,
            removal_plans: self.plans_of_waiting_removals(&waiting)?,
            waiting,
            conflicts,
            notices,
            unpushed,
            unseen_failed: unseen_failed || hidden_conflicts || hidden_notices,
        })
    }

    pub fn seen_staged(&self) -> Result<Vec<Authored>, UseCaseError> {
        let mut seen = Vec::new();
        for mut authored in self.store.staged()? {
            if self.permits_change(Permission::StageRead, &authored.change) {
                authored.change = self.as_seen_change(authored.change)?;
                seen.push(authored);
            }
        }
        Ok(seen)
    }

    pub fn diff_working(&self) -> Result<Vec<Change>, UseCaseError> {
        let mut seen = Vec::new();
        for change in working_changes(self.store)? {
            if self.permits_change(Permission::WorkingRead, &change) {
                seen.push(self.as_seen_change(change)?);
            }
        }
        Ok(seen)
    }
}

fn taken_by(accepted: Option<&[Kind]>, change: &Change) -> bool {
    let Some(kinds) = accepted else {
        return true;
    };
    kinds.contains(&change.object().kind())
}

fn working_changes(store: &dyn Store) -> Result<Vec<Change>, UseCaseError> {
    let staged: BTreeMap<Oid, Change> = store
        .staged()?
        .into_iter()
        .map(|c| (c.change.oid().clone(), c.change))
        .collect();
    let mut out = Vec::new();
    for oid in working_and_ever_committed_oids(store)? {
        let committed_with_the_stage_applied: Option<Object> = match staged.get(&oid) {
            Some(change) => change.after().cloned(),
            None => store.committed(&oid)?,
        };
        let working = store.get(&oid)?;
        if let Some(change) = Change::between(committed_with_the_stage_applied, working) {
            out.push(change);
        }
    }
    Ok(out)
}

mod seen;
mod unpushed;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod token_tests;
