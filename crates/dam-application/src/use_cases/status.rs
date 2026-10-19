use std::collections::{BTreeMap, BTreeSet};

use dam_domain::{Change, CommitId, Kind, Object, Oid};

use crate::errors::UseCaseError;
use crate::ports::{CommitRepository, Conflict, Notice, RemoteName, Store};
use crate::service::Dam;
use crate::use_cases::stage::working_and_ever_committed_oids;

pub struct Status {
    pub staged: Vec<Change>,
    pub unstaged: Vec<Change>,
    pub conflicts: Vec<Conflict>,
    pub notices: Vec<Notice>,
    pub unpushed: Vec<Unpushed>,
}

pub struct Unpushed {
    pub remote: RemoteName,
    pub commit_ids: Vec<CommitId>,
    pub oids: Vec<Oid>,
}

impl Dam<'_> {
    pub fn status(&self, remotes: &[RemoteName]) -> Result<Status, UseCaseError> {
        let mut unpushed = Vec::new();
        for remote in remotes {
            let accepted = self.store.accepted_kinds(remote)?;
            unpushed.push(unpushed_for(self.store, remote, accepted.as_deref())?);
        }
        Ok(Status {
            staged: self.store.staged()?,
            unstaged: working_changes(self.store)?,
            conflicts: self.store.conflicts()?,
            notices: self.store.notices()?,
            unpushed,
        })
    }

    pub fn diff_working(&self) -> Result<Vec<Change>, UseCaseError> {
        working_changes(self.store)
    }
}

fn unpushed_for(
    commits: &dyn CommitRepository,
    remote: &RemoteName,
    accepted: Option<&[Kind]>,
) -> Result<Unpushed, UseCaseError> {
    let mut commit_ids = Vec::new();
    let mut oids = Vec::new();
    let mut named = BTreeSet::new();
    let newest_first = commits.unpushed(remote)?.into_iter().rev();
    for record in newest_first {
        let owed: Vec<&Change> = record
            .changes
            .iter()
            .filter(|change| taken_by(accepted, change))
            .collect();
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
    Ok(Unpushed {
        remote: remote.clone(),
        commit_ids,
        oids,
    })
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
        .map(|c| (c.oid().clone(), c))
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

#[cfg(test)]
mod tests;
