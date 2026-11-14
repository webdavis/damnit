mod refusal_name;

use dam_domain::{Blockers, Date, LabelViolation, Oid, Permission, QueryError, Tracked};

use crate::access::TokenRefusal;
use crate::ports::{CredentialError, HelperError, StoreError};
use crate::use_cases::items::refusal::ItemRefusal;

#[derive(Debug, PartialEq, Eq)]
pub enum Refusal {
    Blocked {
        oid: Oid,
        blockers: Blockers,
    },
    Cycle {
        oid: Oid,
        path: Vec<Oid>,
    },
    Labels(LabelViolation),
    UnknownCategory(String),
    NotACategoryValue {
        category: String,
        value: String,
    },
    NoSuchObject(String),
    NoWorkingObject(String),
    AmbiguousPrefix {
        text: String,
        matches: Vec<String>,
    },
    NoSuchRemote(String),
    RemoteExists(String),
    NotATask(Oid),
    NotAnEvent(Oid),
    NotCompleted(Oid),
    NotCommitted(Oid),
    DirtyOnPull {
        oid: Oid,
    },
    MoveInsideItself(Oid),
    SubjectNamesNoFolder(Oid),
    NothingToCommit,
    NeedsAnAnswer,
    NeedsAnEditor,
    UnresolvedConflicts(Option<usize>),
    MissingCredential {
        remote: String,
        name: String,
    },
    StaleRemote {
        remote: String,
        age: Option<u64>,
        limit: u64,
    },
    UnreadableRecurrence {
        rule: String,
        tracked: Tracked,
    },
    NoConflict(Oid),
    Token(TokenRefusal),
    NotPermitted(Permission),
    NotOpenToTokens(&'static str),
    FocusFull {
        day: Date,
        focused: Vec<Oid>,
    },
    TokenExists(String),
    NotGrantable(String),
    NoSuchToken(String),
    Item(ItemRefusal),
}

#[derive(Debug, PartialEq, Eq)]
pub enum UseCaseError {
    Refused(Refusal),
    Store(StoreError),
    Helper(HelperError),
    PushStopped {
        by: HelperError,
        kept: usize,
        unanswered: usize,
    },
    Credential(CredentialError),
    Parse(String),
    Query(QueryError),
    NotAnOid(String),
}

impl From<Refusal> for UseCaseError {
    fn from(r: Refusal) -> UseCaseError {
        UseCaseError::Refused(r)
    }
}
impl From<ItemRefusal> for UseCaseError {
    fn from(r: ItemRefusal) -> UseCaseError {
        UseCaseError::Refused(Refusal::Item(r))
    }
}
impl From<StoreError> for UseCaseError {
    fn from(e: StoreError) -> UseCaseError {
        UseCaseError::Store(e)
    }
}
impl From<HelperError> for UseCaseError {
    fn from(e: HelperError) -> UseCaseError {
        UseCaseError::Helper(e)
    }
}
impl From<CredentialError> for UseCaseError {
    fn from(e: CredentialError) -> UseCaseError {
        UseCaseError::Credential(e)
    }
}

#[cfg(test)]
mod tests;
