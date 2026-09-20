use dam_domain::{Change, CommitId, CommitRecord, Date, Kind, Object, Oid, Path, Timestamp};

use crate::config::{CredentialSpec, RemoteConfig};
use crate::errors::UseCaseError;
use crate::remote::{MutationOutcome, PullOutcome, RemoteCapabilities, RemoteMutation};
use crate::secret::Secret;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RemoteName(pub String);

#[derive(Debug, PartialEq, Eq)]
pub struct StoreError(pub String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conflict {
    pub remote: RemoteName,
    pub oid: Oid,
    pub ours: Object,
    pub theirs: Object,
}

/// Something upstream did that dam reports and never acts on by itself.
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
    /// Upstream changed an object's kind; ours is kept as it is.
    KindChanged {
        oid: Oid,
        ours: Kind,
        theirs: Kind,
    },
}

/// The working layer: every object as it stands right now, plus the last
/// commit's view of one object for diffing working against committed.
pub trait ObjectRepository {
    fn get(&self, oid: &Oid) -> Result<Option<Object>, StoreError>;
    fn all(&self) -> Result<Vec<Object>, StoreError>;
    fn children_of(&self, path: &Path) -> Result<Vec<Object>, StoreError>;
    fn dependents_of(&self, oid: &Oid) -> Result<Vec<Oid>, StoreError>;
    fn put(&self, object: &Object) -> Result<(), StoreError>;
    fn delete(&self, oid: &Oid) -> Result<(), StoreError>;
    fn committed(&self, oid: &Oid) -> Result<Option<Object>, StoreError>;
}

/// The stage: at most one change per oid, waiting for a commit.
pub trait StageRepository {
    fn stage(&self, change: Change) -> Result<(), StoreError>;
    fn unstage(&self, oid: &Oid) -> Result<(), StoreError>;
    fn unstage_all(&self) -> Result<(), StoreError>;
    fn staged(&self) -> Result<Vec<Change>, StoreError>;
}

/// The commit history, and which of it each remote has taken.
pub trait CommitRepository {
    fn commit(&self, record: &CommitRecord) -> Result<(), StoreError>;
    fn log(&self) -> Result<Vec<CommitRecord>, StoreError>;
    fn unpushed(&self, remote: &RemoteName) -> Result<Vec<CommitRecord>, StoreError>;
    fn mark_pushed(&self, remote: &RemoteName, id: &CommitId) -> Result<(), StoreError>;
    /// Oids a previous push failed on, resent as updates until they succeed.
    fn push_retries(&self, remote: &RemoteName) -> Result<Vec<Oid>, StoreError>;
    fn set_push_retries(&self, remote: &RemoteName, oids: &[Oid]) -> Result<(), StoreError>;
}

/// What each remote holds: the id it knows an object by, the snapshot of what
/// it last sent, and where its incremental sync stands.
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
    /// Drops the remote id and the remote snapshot for one oid on one remote,
    /// for an object the remote no longer has after a successful delete.
    fn clear_remote_mapping(&self, remote: &RemoteName, oid: &Oid) -> Result<(), StoreError>;
    fn sync_token(&self, remote: &RemoteName) -> Result<Option<String>, StoreError>;
    fn set_sync_token(&self, remote: &RemoteName, token: Option<&str>) -> Result<(), StoreError>;
    fn last_pull(&self, remote: &RemoteName) -> Result<Option<Timestamp>, StoreError>;
    fn set_last_pull(&self, remote: &RemoteName, at: Timestamp) -> Result<(), StoreError>;
}

/// Objects a pull could not merge, held until the operator settles them.
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

/// Things upstream did that dam only reports, in the order they were recorded.
pub trait NoticeRepository {
    fn add_notice(&self, notice: &Notice) -> Result<(), StoreError>;
    fn notices(&self) -> Result<Vec<Notice>, StoreError>;
    fn clear_notices(&self) -> Result<(), StoreError>;
}

/// Runs one unit of work so that every write inside it lands together or
/// none of it does, however many record families it spans.
pub trait Transactional {
    fn in_transaction(
        &self,
        work: &mut dyn FnMut() -> Result<(), UseCaseError>,
    ) -> Result<(), UseCaseError>;
}

/// One durable store behind all six record families, which is what the
/// composition root holds and what a transaction spans.
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

/// The repositories a use case that spans several families borrows at once,
/// so it names each family it touches without taking six parameters.
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

#[derive(Debug, PartialEq, Eq)]
pub enum HelperError {
    NotFound {
        helper: String,
    },
    Protocol(String),
    UnsupportedProtocol {
        helper: String,
        found: u32,
        supported: u32,
    },
    Io(String),
    Remote(String),
    Timeout {
        helper: String,
        deadline: String,
    },
    Cancelled,
}

pub trait RemoteHelper: std::fmt::Debug {
    fn capabilities(&mut self) -> Result<RemoteCapabilities, HelperError>;
    fn pull(&mut self, since: Option<&str>) -> Result<PullOutcome, HelperError>;
    fn push(&mut self, mutations: Vec<RemoteMutation>)
    -> Result<Vec<MutationOutcome>, HelperError>;
}

pub trait HelperLauncher {
    fn launch(
        &self,
        remote: &RemoteConfig,
        credentials: &[(String, Secret)],
    ) -> Result<Box<dyn RemoteHelper>, HelperError>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum CredentialError {
    Missing(String),
    CommandFailed { name: String, why: String },
    EnvUnset { name: String, var: String },
}

pub trait CredentialSource {
    fn resolve(&self, spec: &CredentialSpec) -> Result<Secret, CredentialError>;
}

pub trait Clock {
    fn today(&self) -> Date;
    fn now(&self) -> Timestamp;
}

/// A source of entropy for a new identifier. Shared rather than owned, so a
/// use case that mints one borrows it alongside everything else it reads.
pub trait Randomness {
    fn fill(&self, buf: &mut [u8]);
}

#[derive(Debug, PartialEq, Eq)]
pub struct EditorError(pub String);

pub trait EditorSession {
    fn edit(&self, text: &str) -> Result<String, EditorError>;
}
