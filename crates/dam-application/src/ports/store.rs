use dam_domain::{Change, CommitId, CommitRecord, Kind, Object, Oid, Path, Timestamp};

use crate::errors::UseCaseError;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RemoteName(pub String);

#[derive(Debug, PartialEq, Eq)]
pub enum StoreError {
    Busy(String),
    Failed(String),
}

impl StoreError {
    pub fn why(&self) -> &str {
        match self {
            StoreError::Busy(why) | StoreError::Failed(why) => why,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conflict {
    pub remote: RemoteName,
    pub oid: Oid,
    pub ours: Object,
    pub theirs: Object,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Notice {
    RemovedUpstream {
        remote: RemoteName,
        oid: Oid,
        subject: String,
    },
    EventCancelled {
        oid: Oid,
        subject: String,
        attached: usize,
    },
    PushFailed {
        remote: RemoteName,
        oid: Oid,
        why: String,
    },
    PullFailed {
        remote: RemoteName,
        why: String,
    },
    KindChanged {
        oid: Oid,
        ours: Kind,
        theirs: Kind,
    },
}

pub trait ObjectRepository {
    fn get(&self, oid: &Oid) -> Result<Option<Object>, StoreError>;
    fn all(&self) -> Result<Vec<Object>, StoreError>;
    fn children_of(&self, path: &Path) -> Result<Vec<Object>, StoreError>;
    fn dependents_of(&self, oid: &Oid) -> Result<Vec<Oid>, StoreError>;
    fn put(&self, object: &Object) -> Result<(), StoreError>;
    fn delete(&self, oid: &Oid) -> Result<(), StoreError>;
    fn committed(&self, oid: &Oid) -> Result<Option<Object>, StoreError>;
}

pub trait StageRepository {
    fn stage(&self, change: Change) -> Result<(), StoreError>;
    fn unstage(&self, oid: &Oid) -> Result<(), StoreError>;
    fn unstage_all(&self) -> Result<(), StoreError>;
    fn staged(&self) -> Result<Vec<Change>, StoreError>;
}

pub trait CommitRepository {
    fn commit(&self, record: &CommitRecord) -> Result<(), StoreError>;
    fn log(&self) -> Result<Vec<CommitRecord>, StoreError>;
    fn unpushed(&self, remote: &RemoteName) -> Result<Vec<CommitRecord>, StoreError>;
    fn mark_pushed(&self, remote: &RemoteName, id: &CommitId) -> Result<(), StoreError>;
    fn push_retries(&self, remote: &RemoteName) -> Result<Vec<Oid>, StoreError>;
    fn set_push_retries(&self, remote: &RemoteName, oids: &[Oid]) -> Result<(), StoreError>;
}

pub trait RemoteTrackingRepository {
    fn remote_id(&self, remote: &RemoteName, oid: &Oid) -> Result<Option<String>, StoreError>;
    fn oid_for_remote_id(
        &self,
        remote: &RemoteName,
        remote_id: &str,
    ) -> Result<Option<Oid>, StoreError>;
    fn map_remote_id(
        &self,
        remote: &RemoteName,
        oid: &Oid,
        remote_id: &str,
    ) -> Result<(), StoreError>;
    fn remote_snapshot(&self, remote: &RemoteName, oid: &Oid)
    -> Result<Option<Object>, StoreError>;
    fn set_remote_snapshot(&self, remote: &RemoteName, object: &Object) -> Result<(), StoreError>;
    fn clear_remote_mapping(&self, remote: &RemoteName, oid: &Oid) -> Result<(), StoreError>;
    fn sync_token(&self, remote: &RemoteName) -> Result<Option<String>, StoreError>;
    fn set_sync_token(&self, remote: &RemoteName, token: Option<&str>) -> Result<(), StoreError>;
    fn last_pull(&self, remote: &RemoteName) -> Result<Option<Timestamp>, StoreError>;
    fn set_last_pull(&self, remote: &RemoteName, at: Timestamp) -> Result<(), StoreError>;
    fn last_push(&self, remote: &RemoteName) -> Result<Option<Timestamp>, StoreError>;
    fn set_last_push(&self, remote: &RemoteName, at: Timestamp) -> Result<(), StoreError>;
}

pub trait ConflictRepository {
    fn mark_conflict(
        &self,
        remote: &RemoteName,
        oid: &Oid,
        theirs: &Object,
    ) -> Result<(), StoreError>;
    fn conflicts(&self) -> Result<Vec<Conflict>, StoreError>;
    fn clear_conflict(&self, oid: &Oid) -> Result<(), StoreError>;
}

pub trait NoticeRepository {
    fn add_notice(&self, notice: &Notice) -> Result<(), StoreError>;
    fn notices(&self) -> Result<Vec<Notice>, StoreError>;
    fn clear_notices(&self) -> Result<(), StoreError>;
}

pub trait Transactional {
    fn in_transaction(
        &self,
        work: &mut dyn FnMut() -> Result<(), UseCaseError>,
    ) -> Result<(), UseCaseError>;
}

pub trait Store:
    ObjectRepository
    + StageRepository
    + CommitRepository
    + RemoteTrackingRepository
    + ConflictRepository
    + NoticeRepository
    + Transactional
{
}

impl<T> Store for T where
    T: ObjectRepository
        + StageRepository
        + CommitRepository
        + RemoteTrackingRepository
        + ConflictRepository
        + NoticeRepository
        + Transactional
{
}

#[derive(Clone, Copy)]
pub struct Repositories<'a> {
    pub objects: &'a dyn ObjectRepository,
    pub stage: &'a dyn StageRepository,
    pub commits: &'a dyn CommitRepository,
    pub remote_tracking: &'a dyn RemoteTrackingRepository,
    pub conflicts: &'a dyn ConflictRepository,
    pub notices: &'a dyn NoticeRepository,
    pub transaction: &'a dyn Transactional,
}

impl<'a> Repositories<'a> {
    pub fn of(store: &'a dyn Store) -> Repositories<'a> {
        Repositories {
            objects: store,
            stage: store,
            commits: store,
            remote_tracking: store,
            conflicts: store,
            notices: store,
            transaction: store,
        }
    }
}
