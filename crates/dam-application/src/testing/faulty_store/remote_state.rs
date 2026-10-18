use dam_domain::{Object, Oid, Timestamp};

use crate::ports::{
    Conflict, ConflictRepository, Notice, NoticeRepository, RemoteName, RemoteTrackingRepository,
    StoreError,
};

use super::FaultyStore;

impl RemoteTrackingRepository for FaultyStore {
    fn remote_id(&self, remote: &RemoteName, oid: &Oid) -> Result<Option<String>, StoreError> {
        self.inner.remote_id(remote, oid)
    }
    fn oid_for_remote_id(
        &self,
        remote: &RemoteName,
        remote_id: &str,
    ) -> Result<Option<Oid>, StoreError> {
        self.inner.oid_for_remote_id(remote, remote_id)
    }
    fn map_remote_id(
        &self,
        remote: &RemoteName,
        oid: &Oid,
        remote_id: &str,
    ) -> Result<(), StoreError> {
        self.write()?;
        self.inner.map_remote_id(remote, oid, remote_id)
    }
    fn remote_snapshot(
        &self,
        remote: &RemoteName,
        oid: &Oid,
    ) -> Result<Option<Object>, StoreError> {
        self.inner.remote_snapshot(remote, oid)
    }
    fn set_remote_snapshot(&self, remote: &RemoteName, object: &Object) -> Result<(), StoreError> {
        self.write()?;
        self.inner.set_remote_snapshot(remote, object)
    }
    fn clear_remote_mapping(&self, remote: &RemoteName, oid: &Oid) -> Result<(), StoreError> {
        self.write()?;
        self.inner.clear_remote_mapping(remote, oid)
    }
    fn sync_token(&self, remote: &RemoteName) -> Result<Option<String>, StoreError> {
        self.inner.sync_token(remote)
    }
    fn set_sync_token(&self, remote: &RemoteName, token: Option<&str>) -> Result<(), StoreError> {
        self.write()?;
        self.inner.set_sync_token(remote, token)
    }
    fn last_pull(&self, remote: &RemoteName) -> Result<Option<Timestamp>, StoreError> {
        self.inner.last_pull(remote)
    }
    fn set_last_pull(&self, remote: &RemoteName, at: Timestamp) -> Result<(), StoreError> {
        self.write()?;
        self.inner.set_last_pull(remote, at)
    }
    fn last_push(&self, remote: &RemoteName) -> Result<Option<Timestamp>, StoreError> {
        self.inner.last_push(remote)
    }
    fn set_last_push(&self, remote: &RemoteName, at: Timestamp) -> Result<(), StoreError> {
        self.write()?;
        self.inner.set_last_push(remote, at)
    }
}

impl ConflictRepository for FaultyStore {
    fn mark_conflict(
        &self,
        remote: &RemoteName,
        oid: &Oid,
        theirs: &Object,
    ) -> Result<(), StoreError> {
        self.write()?;
        self.inner.mark_conflict(remote, oid, theirs)
    }
    fn conflicts(&self) -> Result<Vec<Conflict>, StoreError> {
        self.inner.conflicts()
    }
    fn clear_conflict(&self, oid: &Oid) -> Result<(), StoreError> {
        self.write()?;
        self.inner.clear_conflict(oid)
    }
}

impl NoticeRepository for FaultyStore {
    fn add_notice(&self, notice: &Notice) -> Result<(), StoreError> {
        self.write()?;
        self.inner.add_notice(notice)
    }
    fn notices(&self) -> Result<Vec<Notice>, StoreError> {
        self.inner.notices()
    }
    fn clear_notices(&self) -> Result<(), StoreError> {
        self.write()?;
        self.inner.clear_notices()
    }
}
