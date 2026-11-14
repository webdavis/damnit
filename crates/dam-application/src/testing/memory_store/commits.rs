use dam_domain::{Authored, CommitId, CommitRecord, Oid};

use super::MemoryStore;
use crate::ports::{CommitRepository, RemoteName, StoreError};

impl CommitRepository for MemoryStore {
    fn commit(&self, record: &CommitRecord) -> Result<(), StoreError> {
        let mut inner = self.0.borrow_mut();
        for Authored { change, .. } in &record.changes {
            match change.after() {
                Some(after) => inner.committed.insert(change.oid().clone(), after.clone()),
                None => inner.committed.remove(change.oid()),
            };
            inner.staged.retain(|(staged, _), _| staged != change.oid());
        }
        inner.commits.push(record.clone());
        Ok(())
    }
    fn log(&self) -> Result<Vec<CommitRecord>, StoreError> {
        Ok(self.0.borrow().commits.iter().rev().cloned().collect())
    }
    fn unpushed(&self, remote: &RemoteName) -> Result<Vec<CommitRecord>, StoreError> {
        let inner = self.0.borrow();
        let pushed = inner.pushed.get(remote).cloned().unwrap_or_default();
        Ok(inner
            .commits
            .iter()
            .filter(|c| !pushed.contains(&c.id))
            .cloned()
            .collect())
    }
    fn mark_pushed(&self, remote: &RemoteName, id: &CommitId) -> Result<(), StoreError> {
        self.0
            .borrow_mut()
            .pushed
            .entry(remote.clone())
            .or_default()
            .push(id.clone());
        Ok(())
    }
    fn push_retries(&self, remote: &RemoteName) -> Result<Vec<Oid>, StoreError> {
        Ok(self
            .0
            .borrow()
            .retries
            .get(remote)
            .cloned()
            .unwrap_or_default())
    }
    fn set_push_retries(&self, remote: &RemoteName, oids: &[Oid]) -> Result<(), StoreError> {
        self.0
            .borrow_mut()
            .retries
            .insert(remote.clone(), oids.to_vec());
        Ok(())
    }
}
