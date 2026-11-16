mod filter_error;
mod refusal_name;
mod retry;

pub use filter_error::FilterError;
pub use retry::Retry;

use dam_domain::{
    Blockers, CommitId, Date, LabelViolation, Oid, Path, Permission, QueryError, Tracked,
};

use crate::access::TokenRefusal;
use crate::ports::{CredentialError, HelperError, HookFailure, StoreError};
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
    MoveInsideItself(Oid),
    SubjectNamesNoFolder(Oid),
    NothingToCommit,
    NeedsAnAnswer,
    NeedsAnEditor,
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
    NoSuchSetting(String),
    HookRefused(HookFailure),
    StartedByHook(String),
    Item(ItemRefusal),
    PulledCommit(CommitId),
    NotInCommit {
        commit: CommitId,
        text: String,
    },
    ParentNotKnown {
        oid: Oid,
        unsettled: Vec<Oid>,
        namesakes: Vec<Oid>,
        folder: Path,
        held_by_a_remote: bool,
    },
    PathOutsideParent {
        parent: Oid,
        folder: Path,
        path: Path,
    },
    SeveralHoldThePath {
        namesakes: Vec<Oid>,
        folder: Path,
        retry: Retry,
    },
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
    Filter(FilterError),
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
