use std::fmt;

use dam_domain::{Blocker, LabelViolation, Oid};

use crate::ports::{CredentialError, EditorError, HelperError, StoreError};

mod refusal_display;

/// A rule dam enforces, refused with the reason.
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
    /// A prefix that matched nothing in the working layer, which is the
    /// layer a prefix is resolved against.
    NoWorkingObject(String),
    NoSuchRemote(String),
    NotATask(Oid),
    /// An event field asked of a task, the mirror of `NotATask`.
    NotAnEvent(Oid),
    NotCompleted(Oid),
    NotCommitted(Oid),
    DirtyOnPull {
        oid: Oid,
    },
    /// A path that would put an object under itself, which no tree allows.
    MoveInsideItself(Oid),
    NothingToCommit,
    /// A verb needed the operator's answer and the format cannot carry one.
    NeedsAnAnswer,
    /// `-e` needed an editor and the format cannot open one.
    NeedsAnEditor,
    UnresolvedConflicts(usize),
    MissingCredential {
        remote: String,
        name: String,
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
    /// The rule this refusal broke, one stable snake_case word per variant,
    /// which is what a client branches on when `refused` is too coarse.
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

impl fmt::Display for UseCaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UseCaseError::Refused(r) => write!(f, "{r}"),
            UseCaseError::Store(StoreError::Busy(why)) => {
                write!(f, "the store is busy, another dam is writing it: {why}")
            }
            UseCaseError::Store(StoreError::Failed(why)) => write!(f, "storage: {why}"),
            UseCaseError::Helper(HelperError::NotFound { helper }) => {
                write!(f, "{helper} was not found on PATH")
            }
            UseCaseError::Helper(HelperError::Protocol(s)) => {
                write!(f, "the helper answered something dam cannot read: {s}")
            }
            UseCaseError::Helper(HelperError::UnsupportedProtocol {
                helper,
                found,
                supported,
            }) => write!(
                f,
                "helper {helper} speaks protocol version {found}; dam speaks {supported}, so upgrade dam or use an older helper"
            ),
            UseCaseError::Helper(HelperError::CollidingCredentials {
                first,
                second,
                variable,
            }) => write!(
                f,
                "credentials {first} and {second} both become {variable}; rename one of them"
            ),
            UseCaseError::Helper(HelperError::Io(s)) => write!(f, "talking to the helper: {s}"),
            UseCaseError::Helper(HelperError::Remote(s)) => write!(f, "the remote refused: {s}"),
            UseCaseError::Helper(HelperError::Timeout {
                helper,
                deadline,
                said,
            }) => {
                write!(f, "helper {helper} gave no answer within {deadline}")?;
                match said {
                    Some(text) => write!(f, "; it said: {text}"),
                    None => Ok(()),
                }
            }
            UseCaseError::Helper(HelperError::Cancelled) => f.write_str("cancelled"),
            UseCaseError::Credential(CredentialError::Missing(n)) => {
                write!(f, "credential {n} has no source")
            }
            UseCaseError::Credential(CredentialError::CommandFailed { name, why }) => {
                write!(f, "the command for {name} failed: {why}")
            }
            UseCaseError::Credential(CredentialError::EnvUnset { name, var }) => {
                write!(f, "{name}: the variable {var} is not set")
            }
            UseCaseError::Editor(e) => write!(f, "editor: {}", e.0),
            UseCaseError::Parse(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for UseCaseError {}

#[cfg(test)]
mod tests {
    use super::*;
    use dam_domain::{LabelViolation, Oid};

    fn oid() -> Oid {
        Oid::generate(&mut |x: &mut [u8]| x.fill(1))
    }

    /// How many slots `slot` below hands out. Kept in sync by hand; nothing
    /// forces it to rise when a new variant reuses an existing slot.
    const RULE_COUNT: usize = 18;

    /// Which variant this is. The match is exhaustive, so a rule added to the
    /// enum does not compile until it has an arm here, but that arm may reuse
    /// an existing index. `every_variant_has_a_sample` then catches a variant
    /// whose sample is missing from `every_refusal`; it does not catch a new
    /// variant that reused a slot instead of taking a fresh one.
    fn slot(refusal: &Refusal) -> usize {
        match refusal {
            Refusal::Blocked { .. } => 0,
            Refusal::Cycle { .. } => 1,
            Refusal::Labels(_) => 2,
            Refusal::UnknownCategory(_) => 3,
            Refusal::NoSuchObject(_) => 4,
            Refusal::NoWorkingObject(_) => 5,
            Refusal::NoSuchRemote(_) => 6,
            Refusal::NotATask(_) => 7,
            Refusal::NotAnEvent(_) => 8,
            Refusal::NotCompleted(_) => 9,
            Refusal::NotCommitted(_) => 10,
            Refusal::DirtyOnPull { .. } => 11,
            Refusal::MoveInsideItself(_) => 12,
            Refusal::NothingToCommit => 13,
            Refusal::NeedsAnAnswer => 14,
            Refusal::NeedsAnEditor => 15,
            Refusal::UnresolvedConflicts(_) => 16,
            Refusal::MissingCredential { .. } => 17,
        }
    }

    /// One of every variant, so the checks below see them all.
    fn every_refusal() -> Vec<Refusal> {
        vec![
            Refusal::Blocked {
                oid: oid(),
                blockers: vec![],
            },
            Refusal::Cycle {
                oid: oid(),
                path: vec![],
            },
            Refusal::Labels(LabelViolation::Exclusive {
                category: "effort".into(),
                held: vec![],
            }),
            Refusal::UnknownCategory("effort".into()),
            Refusal::NoSuchObject("abab".into()),
            Refusal::NoWorkingObject("abab".into()),
            Refusal::NoSuchRemote("todoist".into()),
            Refusal::NotATask(oid()),
            Refusal::NotAnEvent(oid()),
            Refusal::NotCompleted(oid()),
            Refusal::NotCommitted(oid()),
            Refusal::DirtyOnPull { oid: oid() },
            Refusal::MoveInsideItself(oid()),
            Refusal::NothingToCommit,
            Refusal::NeedsAnAnswer,
            Refusal::NeedsAnEditor,
            Refusal::UnresolvedConflicts(2),
            Refusal::MissingCredential {
                remote: "todoist".into(),
                name: "api_token".into(),
            },
        ]
    }

    #[test]
    fn every_variant_has_a_sample() {
        let mut slots: Vec<usize> = every_refusal().iter().map(slot).collect();
        slots.sort_unstable();
        slots.dedup();
        assert_eq!(
            slots,
            (0..RULE_COUNT).collect::<Vec<usize>>(),
            "a rule has no sample in every_refusal, so the checks below never see it"
        );
    }

    #[test]
    fn every_rule_has_its_own_name() {
        let refusals = every_refusal();
        let mut names: Vec<&str> = refusals.iter().map(Refusal::name).collect();
        let count = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), count, "two rules share a name: {names:?}");
    }

    #[test]
    fn every_name_is_lower_snake_case_and_says_something() {
        for refusal in every_refusal() {
            let name = refusal.name();
            assert!(!name.is_empty(), "{refusal:?}");
            assert!(
                name.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
                "{name} is not snake_case"
            );
        }
    }
}
