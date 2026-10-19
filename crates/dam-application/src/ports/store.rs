use std::fmt;

use dam_domain::{Change, CommitId, CommitRecord, Kind, Object, Oid, Path, Timestamp};

use crate::errors::UseCaseError;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RemoteName(pub String);

#[derive(Debug, PartialEq, Eq)]
pub enum RemoteNameError {
    Empty,
    OutsideTheAlphabet(String),
}

impl RemoteName {
    pub fn parse(text: &str) -> Result<RemoteName, RemoteNameError> {
        if text.is_empty() {
            return Err(RemoteNameError::Empty);
        }
        if !text.chars().all(is_a_toml_bare_key_character) {
            return Err(RemoteNameError::OutsideTheAlphabet(text.to_string()));
        }
        Ok(RemoteName(text.to_string()))
    }
}

fn is_a_toml_bare_key_character(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '_'
}

impl fmt::Display for RemoteNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RemoteNameError::Empty => f.write_str("a remote name cannot be empty"),
            RemoteNameError::OutsideTheAlphabet(name) => {
                write!(
                    f,
                    "remote name {name:?} may hold only letters, digits, - and _"
                )
            }
        }
    }
}

impl std::error::Error for RemoteNameError {}

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

pub trait ObjectRepository: Transactional {
    fn get(&self, oid: &Oid) -> Result<Option<Object>, StoreError>;
    fn all(&self) -> Result<Vec<Object>, StoreError>;
    fn children_of(&self, path: &Path) -> Result<Vec<Object>, StoreError>;
    fn dependents_of(&self, oid: &Oid) -> Result<Vec<Oid>, StoreError>;
    fn put(&self, object: &Object) -> Result<(), StoreError>;
    fn delete(&self, oid: &Oid) -> Result<(), StoreError>;
    fn committed(&self, oid: &Oid) -> Result<Option<Object>, StoreError>;
}

pub trait StageRepository {
    fn put_staged(&self, change: Change) -> Result<(), StoreError>;
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
    fn mark_conflict(&self, conflict: &Conflict) -> Result<(), StoreError>;
    fn conflicts(&self) -> Result<Vec<Conflict>, StoreError>;
    fn clear_conflict(&self, oid: &Oid) -> Result<(), StoreError>;
}

pub trait Transactional {
    fn in_transaction(
        &self,
        work: &mut dyn FnMut() -> Result<(), UseCaseError>,
    ) -> Result<(), UseCaseError>;
}

pub trait AcceptedKindsRepository {
    fn accepted_kinds(&self, remote: &RemoteName) -> Result<Option<Vec<Kind>>, StoreError>;
    fn set_accepted_kinds(&self, remote: &RemoteName, kinds: &[Kind]) -> Result<(), StoreError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_remote_name_is_a_toml_bare_key() {
        assert_eq!(
            RemoteName::parse("to-do_1"),
            Ok(RemoteName("to-do_1".into()))
        );
        assert_eq!(
            RemoteName::parse("a.b").unwrap_err().to_string(),
            "remote name \"a.b\" may hold only letters, digits, - and _"
        );
        for bad in ["bad name", "x]", "a\nb", "caf\u{e9}"] {
            assert_eq!(
                RemoteName::parse(bad),
                Err(RemoteNameError::OutsideTheAlphabet(bad.into()))
            );
        }
        assert_eq!(RemoteName::parse(""), Err(RemoteNameError::Empty));
    }
}
