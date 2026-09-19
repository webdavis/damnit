use dam_domain::{Change, CommitId, CommitRecord, Date, Object, Oid, Path, Timestamp};
use dam_protocol::{Capabilities, Mutation, PullResponse, PushResponse};

use crate::config::{CredentialSpec, RemoteConfig};

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
}

pub trait ObjectStore {
    // working layer
    fn get(&self, oid: &Oid) -> Result<Option<Object>, StoreError>;
    fn all(&self) -> Result<Vec<Object>, StoreError>;
    fn children_of(&self, path: &Path) -> Result<Vec<Object>, StoreError>;
    fn dependents_of(&self, oid: &Oid) -> Result<Vec<Oid>, StoreError>;
    fn put(&self, object: &Object) -> Result<(), StoreError>;
    fn delete(&self, oid: &Oid) -> Result<(), StoreError>;
    /// The last commit (HEAD) view of one object, for diffing working against committed.
    fn committed(&self, oid: &Oid) -> Result<Option<Object>, StoreError>;
    // stage
    fn stage(&self, change: Change) -> Result<(), StoreError>;
    fn unstage(&self, oid: &Oid) -> Result<(), StoreError>;
    fn unstage_all(&self) -> Result<(), StoreError>;
    fn staged(&self) -> Result<Vec<Change>, StoreError>;
    // commits
    fn commit(&self, record: &CommitRecord) -> Result<(), StoreError>;
    fn log(&self) -> Result<Vec<CommitRecord>, StoreError>;
    fn unpushed(&self, remote: &RemoteName) -> Result<Vec<CommitRecord>, StoreError>;
    fn mark_pushed(&self, remote: &RemoteName, id: &CommitId) -> Result<(), StoreError>;
    /// Oids a previous push failed on, resent as updates until they succeed.
    fn push_retries(&self, remote: &RemoteName) -> Result<Vec<Oid>, StoreError>;
    fn set_push_retries(&self, remote: &RemoteName, oids: &[Oid]) -> Result<(), StoreError>;
    // remote tracking
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
    // conflicts and notices
    fn mark_conflict(
        &self,
        remote: &RemoteName,
        oid: &Oid,
        theirs: &Object,
    ) -> Result<(), StoreError>;
    fn conflicts(&self) -> Result<Vec<Conflict>, StoreError>;
    fn clear_conflict(&self, oid: &Oid) -> Result<(), StoreError>;
    fn add_notice(&self, notice: &Notice) -> Result<(), StoreError>;
    fn notices(&self) -> Result<Vec<Notice>, StoreError>;
    fn clear_notices(&self) -> Result<(), StoreError>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum HelperError {
    NotFound { helper: String },
    Protocol(String),
    Io(String),
    Remote(String),
    Timeout { helper: String, deadline: String },
    Cancelled,
}

pub trait RemoteHelper: std::fmt::Debug {
    fn capabilities(&mut self) -> Result<Capabilities, HelperError>;
    fn pull(&mut self, since: Option<&str>) -> Result<PullResponse, HelperError>;
    fn push(&mut self, mutations: Vec<Mutation>) -> Result<PushResponse, HelperError>;
}

pub trait HelperLauncher {
    fn launch(
        &self,
        remote: &RemoteConfig,
        credentials: &[(String, String)],
    ) -> Result<Box<dyn RemoteHelper>, HelperError>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum CredentialError {
    Missing(String),
    CommandFailed { name: String, why: String },
    EnvUnset { name: String, var: String },
}

pub trait CredentialSource {
    fn resolve(&self, spec: &CredentialSpec) -> Result<String, CredentialError>;
}

pub trait Clock {
    fn today(&self) -> Date;
    fn now(&self) -> Timestamp;
}

pub trait Randomness {
    fn fill(&mut self, buf: &mut [u8]);
}

#[derive(Debug, PartialEq, Eq)]
pub struct EditorError(pub String);

pub trait EditorSession {
    fn edit(&self, text: &str) -> Result<String, EditorError>;
}
