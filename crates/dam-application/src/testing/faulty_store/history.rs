use dam_domain::{Change, CommitId, CommitRecord, Kind, Oid};

use crate::ports::{
    AcceptedKindsRepository, CommitRepository, RemoteName, StageRepository, StoreError,
};

use super::FaultyStore;

impl StageRepository for FaultyStore {
    fn stage(&self, change: Change) -> Result<(), StoreError> {
        self.write()?;
        self.inner.stage(change)
    }
    fn unstage(&self, oid: &Oid) -> Result<(), StoreError> {
        self.write()?;
        self.inner.unstage(oid)
    }
    fn unstage_all(&self) -> Result<(), StoreError> {
        self.write()?;
        self.inner.unstage_all()
    }
    fn staged(&self) -> Result<Vec<Change>, StoreError> {
        self.inner.staged()
    }
}

impl CommitRepository for FaultyStore {
    fn commit(&self, record: &CommitRecord) -> Result<(), StoreError> {
        self.write()?;
        self.inner.commit(record)
    }
    fn log(&self) -> Result<Vec<CommitRecord>, StoreError> {
        self.inner.log()
    }
    fn unpushed(&self, remote: &RemoteName) -> Result<Vec<CommitRecord>, StoreError> {
        self.inner.unpushed(remote)
    }
    fn mark_pushed(&self, remote: &RemoteName, id: &CommitId) -> Result<(), StoreError> {
        self.write()?;
        self.inner.mark_pushed(remote, id)
    }
    fn push_retries(&self, remote: &RemoteName) -> Result<Vec<Oid>, StoreError> {
        self.inner.push_retries(remote)
    }
    fn set_push_retries(&self, remote: &RemoteName, oids: &[Oid]) -> Result<(), StoreError> {
        self.write()?;
        self.inner.set_push_retries(remote, oids)
    }
}

impl AcceptedKindsRepository for FaultyStore {
    fn accepted_kinds(&self, remote: &RemoteName) -> Result<Option<Vec<Kind>>, StoreError> {
        self.inner.accepted_kinds(remote)
    }
    fn set_accepted_kinds(&self, remote: &RemoteName, kinds: &[Kind]) -> Result<(), StoreError> {
        self.inner.set_accepted_kinds(remote, kinds)
    }
}
