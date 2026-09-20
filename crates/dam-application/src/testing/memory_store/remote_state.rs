//! What syncing with a remote leaves in the double: the oid-to-remote-id
//! mapping and its snapshots, the conflicts a pull raised, and the notices.

use dam_domain::{Object, Oid, Timestamp};

use crate::ports::{
    Conflict, ConflictRepository, Notice, NoticeRepository, ObjectRepository, RemoteName,
    RemoteTrackingRepository, StoreError,
};

use super::MemoryStore;

impl RemoteTrackingRepository for MemoryStore {
    fn remote_id(&self, remote: &RemoteName, oid: &Oid) -> Result<Option<String>, StoreError> {
        Ok(self
            .0
            .borrow()
            .remote_ids
            .get(&(remote.clone(), oid.clone()))
            .cloned())
    }
    fn oid_for_remote_id(
        &self,
        remote: &RemoteName,
        remote_id: &str,
    ) -> Result<Option<Oid>, StoreError> {
        Ok(self
            .0
            .borrow()
            .remote_ids
            .iter()
            .find(|((r, _), id)| r == remote && id.as_str() == remote_id)
            .map(|((_, o), _)| o.clone()))
    }
    fn map_remote_id(
        &self,
        remote: &RemoteName,
        oid: &Oid,
        remote_id: &str,
    ) -> Result<(), StoreError> {
        self.0
            .borrow_mut()
            .remote_ids
            .insert((remote.clone(), oid.clone()), remote_id.to_string());
        Ok(())
    }
    fn remote_snapshot(
        &self,
        remote: &RemoteName,
        oid: &Oid,
    ) -> Result<Option<Object>, StoreError> {
        Ok(self
            .0
            .borrow()
            .snapshots
            .get(&(remote.clone(), oid.clone()))
            .cloned())
    }
    fn set_remote_snapshot(&self, remote: &RemoteName, object: &Object) -> Result<(), StoreError> {
        self.0
            .borrow_mut()
            .snapshots
            .insert((remote.clone(), object.oid().clone()), object.clone());
        Ok(())
    }
    fn clear_remote_mapping(&self, remote: &RemoteName, oid: &Oid) -> Result<(), StoreError> {
        let mut inner = self.0.borrow_mut();
        inner.remote_ids.remove(&(remote.clone(), oid.clone()));
        inner.snapshots.remove(&(remote.clone(), oid.clone()));
        Ok(())
    }
    fn sync_token(&self, remote: &RemoteName) -> Result<Option<String>, StoreError> {
        Ok(self.0.borrow().sync.get(remote).cloned())
    }
    fn set_sync_token(&self, remote: &RemoteName, token: Option<&str>) -> Result<(), StoreError> {
        let mut inner = self.0.borrow_mut();
        match token {
            Some(t) => inner.sync.insert(remote.clone(), t.to_string()),
            None => inner.sync.remove(remote),
        };
        Ok(())
    }
    fn last_pull(&self, remote: &RemoteName) -> Result<Option<Timestamp>, StoreError> {
        Ok(self.0.borrow().last_pulls.get(remote).copied())
    }
    fn set_last_pull(&self, remote: &RemoteName, at: Timestamp) -> Result<(), StoreError> {
        self.0.borrow_mut().last_pulls.insert(remote.clone(), at);
        Ok(())
    }
}

impl ConflictRepository for MemoryStore {
    fn mark_conflict(
        &self,
        remote: &RemoteName,
        oid: &Oid,
        theirs: &Object,
    ) -> Result<(), StoreError> {
        let ours = self
            .get(oid)?
            .ok_or_else(|| StoreError::Failed("conflict on a missing object".into()))?;
        self.0.borrow_mut().conflicts.insert(
            oid.clone(),
            Conflict {
                remote: remote.clone(),
                oid: oid.clone(),
                ours,
                theirs: theirs.clone(),
            },
        );
        Ok(())
    }
    fn conflicts(&self) -> Result<Vec<Conflict>, StoreError> {
        Ok(self.0.borrow().conflicts.values().cloned().collect())
    }
    fn clear_conflict(&self, oid: &Oid) -> Result<(), StoreError> {
        self.0.borrow_mut().conflicts.remove(oid);
        Ok(())
    }
}

impl NoticeRepository for MemoryStore {
    fn add_notice(&self, notice: &Notice) -> Result<(), StoreError> {
        self.0.borrow_mut().notices.push(notice.clone());
        Ok(())
    }
    fn notices(&self) -> Result<Vec<Notice>, StoreError> {
        Ok(self.0.borrow().notices.clone())
    }
    fn clear_notices(&self) -> Result<(), StoreError> {
        self.0.borrow_mut().notices.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::MemoryStore;
    use crate::testing::contract;

    #[test]
    fn it_keeps_the_object_repository_contract() {
        contract::object_repository_contract(&MemoryStore::new());
    }

    #[test]
    fn it_keeps_the_stage_repository_contract() {
        contract::stage_repository_contract(&MemoryStore::new());
    }

    #[test]
    fn it_keeps_the_commit_repository_contract() {
        contract::commit_repository_contract(&MemoryStore::new());
    }

    #[test]
    fn it_keeps_the_remote_tracking_repository_contract() {
        contract::remote_tracking_repository_contract(&MemoryStore::new());
    }

    #[test]
    fn it_keeps_the_conflict_repository_contract() {
        let store = MemoryStore::new();
        contract::conflict_repository_contract(&store, &store);
    }

    #[test]
    fn it_keeps_the_notice_repository_contract() {
        contract::notice_repository_contract(&MemoryStore::new());
    }

    #[test]
    fn it_keeps_the_transactional_contract() {
        let store = MemoryStore::new();
        contract::transactional_contract(&store, &store, &store);
    }
}
