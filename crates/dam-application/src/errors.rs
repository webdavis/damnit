use dam_domain::{Blocker, LabelViolation, Oid};

use crate::ports::{CredentialError, EditorError, HelperError, StoreError};

mod refusal_display;
mod use_case_display;

#[derive(Debug, PartialEq, Eq)]
pub enum Refusal {
    Blocked {
        oid: Oid,
        blockers: Vec<Blocker>,
    },
    Cycle {
        oid: Oid,
        path: Vec<Oid>,
    },
    Labels(LabelViolation),
    UnknownCategory(String),
    NoSuchObject(String),
    NoWorkingObject(String),
    NoSuchRemote(String),
    NotATask(Oid),
    NotAnEvent(Oid),
    NotCompleted(Oid),
    NotCommitted(Oid),
    DirtyOnPull {
        oid: Oid,
    },
    MoveInsideItself(Oid),
    NothingToCommit,
    NeedsAnAnswer,
    NeedsAnEditor,
    UnresolvedConflicts(usize),
    MissingCredential {
        remote: String,
        name: String,
    },
    StaleRemote {
        remote: String,
        age: Option<u64>,
        limit: u64,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum UseCaseError {
    Refused(Refusal),
    Store(StoreError),
    Helper(HelperError),
    Credential(CredentialError),
    Editor(EditorError),
    Parse(String),
}

impl Refusal {
    pub fn name(&self) -> &'static str {
        match self {
            Refusal::Blocked { .. } => "blocked",
            Refusal::Cycle { .. } => "cycle",
            Refusal::Labels(_) => "exclusive_label",
            Refusal::UnknownCategory(_) => "unknown_category",
            Refusal::NoSuchObject(_) => "no_such_object",
            Refusal::NoWorkingObject(_) => "no_working_object",
            Refusal::NoSuchRemote(_) => "no_such_remote",
            Refusal::NotATask(_) => "not_a_task",
            Refusal::NotAnEvent(_) => "not_an_event",
            Refusal::NotCompleted(_) => "not_completed",
            Refusal::NotCommitted(_) => "not_committed",
            Refusal::DirtyOnPull { .. } => "dirty_on_pull",
            Refusal::MoveInsideItself(_) => "move_inside_itself",
            Refusal::NothingToCommit => "nothing_to_commit",
            Refusal::NeedsAnAnswer => "needs_an_answer",
            Refusal::NeedsAnEditor => "needs_an_editor",
            Refusal::UnresolvedConflicts(_) => "unresolved_conflicts",
            Refusal::MissingCredential { .. } => "missing_credential",
            Refusal::StaleRemote { .. } => "stale_remote",
        }
    }
}

impl From<Refusal> for UseCaseError {
    fn from(r: Refusal) -> UseCaseError {
        UseCaseError::Refused(r)
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
impl From<EditorError> for UseCaseError {
    fn from(e: EditorError) -> UseCaseError {
        UseCaseError::Editor(e)
    }
}

impl std::error::Error for UseCaseError {}

#[cfg(test)]
mod tests;
