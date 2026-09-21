use std::fmt;

use dam_domain::{Blocker, LabelViolation, Oid};

use crate::ports::{CredentialError, EditorError, HelperError, StoreError};

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
    NotCompleted(Oid),
    NotCommitted(Oid),
    DirtyOnPull {
        oid: Oid,
    },
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

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refusal::Blocked { oid, blockers } => {
                writeln!(f, "{} cannot be completed:", oid.short())?;
                for b in blockers {
                    match b {
                        Blocker::OpenDependency(d) => {
                            writeln!(f, "  depends on {} which is open", d.short())?
                        }
                        Blocker::OpenChild(c) => writeln!(f, "  child {} is open", c.short())?,
                    }
                }
                f.write_str("use --force to complete it anyway, or --force --interactive to decide what happens to them")
            }
            Refusal::Cycle { oid, path } => {
                let chain: Vec<&str> = path.iter().map(|o| o.short()).collect();
                write!(
                    f,
                    "{} cannot depend on that: it would form a cycle through {}",
                    oid.short(),
                    chain.join(" -> ")
                )
            }
            Refusal::Labels(v) => write!(f, "{v}"),
            Refusal::UnknownCategory(n) => write!(f, "{n:?} is not a declared category"),
            Refusal::NoSuchObject(s) => write!(f, "no object matches {s:?}"),
            Refusal::NoWorkingObject(s) => write!(
                f,
                "no object in the working layer matches {s:?}; one removed from it is named by its full oid"
            ),
            Refusal::NoSuchRemote(s) => write!(f, "no remote named {s:?}"),
            Refusal::NotATask(o) => {
                write!(f, "{} is an event; events are not completed", o.short())
            }
            Refusal::NotCompleted(o) => write!(
                f,
                "{} is not completed, so there is nothing to reopen",
                o.short()
            ),
            Refusal::NotCommitted(o) => write!(
                f,
                "{} has no commit behind it; dam rm removes it instead",
                o.short()
            ),
            Refusal::DirtyOnPull { oid } => write!(
                f,
                "{} changed upstream and has uncommitted local changes; commit or reset it, then pull again",
                oid.short()
            ),
            Refusal::UnresolvedConflicts(n) => {
                write!(f, "{n} conflicts are unresolved; run dam resolve")
            }
            Refusal::MissingCredential { remote, name } => {
                write!(
                    f,
                    "remote {remote:?} needs {name}; set {name}, {name}_command or {name}_env in its config"
                )
            }
        }
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
