mod conversions;
mod filter_error;
mod refusal_name;
mod retry;

pub use filter_error::FilterError;
pub use retry::Retry;

use dam_domain::{
    Blockers, CommitId, Date, LabelViolation, Oid, Path, Permission, QueryError, Tracked,
};

use crate::access::TokenRefusal;
use crate::config::HookEvent;
use crate::ports::{CredentialError, HelperError, HookFailure, ScheduleError, StoreError};
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
    ExclusiveAfterMigrate {
        category: String,
        oids: Vec<Oid>,
    },
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
    SyncNotAllowed(String),
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
    HookNotDeclared(HookEvent),
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
    NoSuchFilter(String),
    CountMismatch {
        expected: usize,
        count: usize,
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
    Schedule(ScheduleError),
}

#[cfg(test)]
mod tests;
