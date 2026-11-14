use std::cell::RefCell;
use std::collections::BTreeMap;

use dam_domain::{
    Author, Authored, CommitId, CommitRecord, IdKey, Item, Kind, Object, Oid, Timestamp,
};

use crate::errors::UseCaseError;
use crate::ports::{
    Conflict, Notice, ObjectRepository, RemoteName, StageRepository, StoreError, Transactional,
};

#[derive(Clone, Debug, Default, PartialEq)]
struct Inner {
    working: BTreeMap<Oid, Object>,
    committed: BTreeMap<Oid, Object>,
    staged: BTreeMap<(Oid, bool), Authored>,
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
    accepted_kinds: BTreeMap<RemoteName, Vec<Kind>>,
    days: BTreeMap<dam_domain::Date, dam_domain::Day>,
    items: Vec<Item>,
    ids: BTreeMap<IdKey, Oid>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MemoryStore(RefCell<Inner>);

impl MemoryStore {
    pub fn new() -> MemoryStore {
        MemoryStore::default()
    }
}

impl Transactional for MemoryStore {
    fn in_transaction(
        &self,
        work: &mut dyn FnMut() -> Result<(), UseCaseError>,
    ) -> Result<(), UseCaseError> {
        let restored_if_the_work_fails = self.0.borrow().clone();
        match work() {
            Ok(()) => Ok(()),
            Err(e) => {
                *self.0.borrow_mut() = restored_if_the_work_fails;
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
    fn put_staged(&self, change: Authored) -> Result<(), StoreError> {
        self.0.borrow_mut().staged.insert(
            (change.change.oid().clone(), change.author == Author::Owner),
            change,
        );
        Ok(())
    }
    fn unstage(&self, oid: &Oid) -> Result<(), StoreError> {
        self.0
            .borrow_mut()
            .staged
            .retain(|(staged, _), _| staged != oid);
        Ok(())
    }
    fn unstage_all(&self) -> Result<(), StoreError> {
        self.0.borrow_mut().staged.clear();
        Ok(())
    }
    fn staged(&self) -> Result<Vec<Authored>, StoreError> {
        Ok(self.0.borrow().staged.values().cloned().collect())
    }
}

mod accepted_kinds;
mod commits;
mod days;
mod ids;
mod items;
mod origins;
mod remote_state;
