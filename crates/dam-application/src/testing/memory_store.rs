//! An in-memory `Store`, the double every application use-case test runs on.

use std::cell::RefCell;
use std::collections::BTreeMap;

use dam_domain::{Change, CommitId, CommitRecord, Object, Oid, Path, Timestamp, coalesce};

use crate::errors::UseCaseError;
use crate::ports::{
    CommitRepository, Conflict, Notice, ObjectRepository, RemoteName, StageRepository, StoreError,
    Transactional,
};

#[derive(Clone, Default)]
struct Inner {
    working: BTreeMap<Oid, Object>,
    committed: BTreeMap<Oid, Object>,
    staged: BTreeMap<Oid, Change>,
    commits: Vec<CommitRecord>,
    pushed: BTreeMap<RemoteName, Vec<CommitId>>,
    remote_ids: BTreeMap<(RemoteName, Oid), String>,
    snapshots: BTreeMap<(RemoteName, Oid), Object>,
    sync: BTreeMap<RemoteName, String>,
    conflicts: BTreeMap<Oid, Conflict>,
    notices: Vec<Notice>,
    retries: BTreeMap<RemoteName, Vec<Oid>>,
    last_pulls: BTreeMap<RemoteName, Timestamp>,
    last_pushes: BTreeMap<RemoteName, Timestamp>,
}

#[derive(Default)]
pub struct MemoryStore(RefCell<Inner>);

impl MemoryStore {
    pub fn new() -> MemoryStore {
        MemoryStore::default()
    }
}

impl Transactional for MemoryStore {
    /// Keeps a copy of everything and puts it back when the work fails, so
    /// the double is as atomic as the durable store it stands in for.
    fn in_transaction(
        &self,
        work: &mut dyn FnMut() -> Result<(), UseCaseError>,
    ) -> Result<(), UseCaseError> {
        let before = self.0.borrow().clone();
        match work() {
            Ok(()) => Ok(()),
            Err(e) => {
                *self.0.borrow_mut() = before;
                Err(e)
            }
        }
    }
}

impl ObjectRepository for MemoryStore {
    fn get(&self, oid: &Oid) -> Result<Option<Object>, StoreError> {
        Ok(self.0.borrow().working.get(oid).cloned())
    }
    fn all(&self) -> Result<Vec<Object>, StoreError> {
        Ok(self.0.borrow().working.values().cloned().collect())
    }
    fn children_of(&self, path: &Path) -> Result<Vec<Object>, StoreError> {
        Ok(self
            .0
            .borrow()
            .working
            .values()
            .filter(|o| o.base().path.parent().as_ref() == Some(path))
            .cloned()
            .collect())
    }
    fn dependents_of(&self, oid: &Oid) -> Result<Vec<Oid>, StoreError> {
        Ok(self
            .0
            .borrow()
            .working
            .values()
            .filter(|o| o.base().depends.contains(oid))
            .map(|o| o.oid().clone())
            .collect())
    }
    fn put(&self, object: &Object) -> Result<(), StoreError> {
        self.0
            .borrow_mut()
            .working
            .insert(object.oid().clone(), object.clone());
        Ok(())
    }
    fn delete(&self, oid: &Oid) -> Result<(), StoreError> {
        self.0.borrow_mut().working.remove(oid);
        Ok(())
    }
    fn committed(&self, oid: &Oid) -> Result<Option<Object>, StoreError> {
        Ok(self.0.borrow().committed.get(oid).cloned())
    }
}

impl StageRepository for MemoryStore {
    fn stage(&self, change: Change) -> Result<(), StoreError> {
        let mut inner = self.0.borrow_mut();
        let oid = change.oid.clone();
        let merged = match inner.staged.remove(&oid) {
            Some(existing) => coalesce(existing, change),
            None => Some(change),
        };
        if let Some(c) = merged {
            inner.staged.insert(oid, c);
        }
        Ok(())
    }
    fn unstage(&self, oid: &Oid) -> Result<(), StoreError> {
        self.0.borrow_mut().staged.remove(oid);
        Ok(())
    }
    fn unstage_all(&self) -> Result<(), StoreError> {
        self.0.borrow_mut().staged.clear();
        Ok(())
    }
    fn staged(&self) -> Result<Vec<Change>, StoreError> {
        Ok(self.0.borrow().staged.values().cloned().collect())
    }
}

impl CommitRepository for MemoryStore {
    fn commit(&self, record: &CommitRecord) -> Result<(), StoreError> {
        let mut inner = self.0.borrow_mut();
        for change in &record.changes {
            match &change.after {
                Some(after) => inner.committed.insert(change.oid.clone(), after.clone()),
                None => inner.committed.remove(&change.oid),
            };
            inner.staged.remove(&change.oid);
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

mod remote_state;
