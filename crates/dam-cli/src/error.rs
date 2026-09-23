use std::fmt;

use dam_adapters::{ConfigError, OpenError};
use dam_application::{Refusal, StoreError, UseCaseError};
use dam_domain::{Blocker, Oid};

#[derive(Debug)]
pub(crate) enum CliError {
    UseCase(UseCaseError),
    Config(ConfigError),
    Open(OpenError),
    Usage(String),
    /// Constructed by `oids::resolve_oid` when a prefix or path names more than one object.
    Ambiguous {
        text: String,
        matches: Vec<Oid>,
    },
    /// The operator interrupted, a prompt hit EOF, or an answer was unscripted.
    Cancelled,
    Io(String),
}

impl CliError {
    /// One meaning per code, so a client can tell what happened:
    ///
    /// | 0 | the command did what it was asked |
    /// | 1 | dam failed: a store, config, helper, editor or io failure |
    /// | 2 | the command line was wrong, which is also clap's own code |
    /// | 3 | cancelled: an interrupt, or a prompt with no answer |
    /// | 4 | dam refused by one of its own rules |
    pub(crate) fn exit_code(&self) -> i32 {
        match self {
            CliError::Usage(_) | CliError::Ambiguous { .. } => 2,
            CliError::Cancelled => 3,
            CliError::UseCase(UseCaseError::Refused(_)) => 4,
            _ => 1,
        }
    }

    /// The failure class, one stable word per arm, which is what a client
    /// branches on instead of reading the message.
    pub(crate) fn kind(&self) -> &'static str {
        match self {
            CliError::UseCase(UseCaseError::Refused(_)) => "refused",
            CliError::UseCase(UseCaseError::Store(_))
            | CliError::Open(_)
            | CliError::Io(_)
            | CliError::Config(ConfigError::Io(_)) => "store",
            CliError::UseCase(UseCaseError::Helper(_)) => "helper",
            CliError::UseCase(UseCaseError::Credential(_)) => "credential",
            // Only a human run opens an editor, so this classifies a failure
            // no document is ever printed for.
            CliError::UseCase(UseCaseError::Editor(_)) => "editor",
            CliError::UseCase(UseCaseError::Parse(_)) | CliError::Config(_) => "parse",
            CliError::Usage(_) | CliError::Ambiguous { .. } => "usage",
            CliError::Cancelled => "cancelled",
        }
    }

    /// The objects the message names, in full and in the order it names them:
    /// for a blocked completion the task, then each blocker.
    pub(crate) fn oids(&self) -> Vec<String> {
        match self {
            CliError::UseCase(UseCaseError::Refused(refusal)) => refusal_oids(refusal),
            CliError::Ambiguous { matches, .. } => matches.iter().map(Oid::to_string).collect(),
            _ => Vec::new(),
        }
    }

    /// The rule a refusal broke, and null for every other failure, so a
    /// client can tell a blocked completion from a missing object without
    /// reading the sentence.
    pub(crate) fn rule(&self) -> Option<&'static str> {
        match self {
            CliError::UseCase(UseCaseError::Refused(refusal)) => Some(refusal.name()),
            _ => None,
        }
    }

    /// The whole failure as one document, which `--json` and `--toon` print on
    /// standard error in place of the plain line.
    pub(crate) fn document(&self) -> serde_json::Value {
        serde_json::json!({
            "error": {
                "kind": self.kind(),
                "rule": self.rule(),
                "message": self.to_string(),
                "oids": self.oids(),
            }
        })
    }
}

fn refusal_oids(refusal: &Refusal) -> Vec<String> {
    let named = |oid: &Oid| oid.to_string();
    match refusal {
        Refusal::Blocked { oid, blockers } => std::iter::once(named(oid))
            .chain(blockers.iter().map(|b| named(blocked_by(b))))
            .collect(),
        Refusal::Cycle { oid, path } => std::iter::once(named(oid))
            .chain(path.iter().map(named))
            .collect(),
        Refusal::NotATask(oid)
        | Refusal::NotAnEvent(oid)
        | Refusal::MoveInsideItself(oid)
        | Refusal::NotCompleted(oid)
        | Refusal::NotCommitted(oid)
        | Refusal::DirtyOnPull { oid } => vec![named(oid)],
        Refusal::Labels(_)
        | Refusal::UnknownCategory(_)
        | Refusal::NoSuchObject(_)
        | Refusal::NoWorkingObject(_)
        | Refusal::NoSuchRemote(_)
        | Refusal::UnresolvedConflicts(_)
        | Refusal::MissingCredential { .. }
        | Refusal::StaleRemote { .. }
        | Refusal::NothingToCommit
        | Refusal::NeedsAnAnswer
        | Refusal::NeedsAnEditor => Vec::new(),
    }
}

fn blocked_by(blocker: &Blocker) -> &Oid {
    match blocker {
        Blocker::OpenDependency(oid) | Blocker::OpenChild(oid) => oid,
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::UseCase(e) => write!(f, "{e}"),
            CliError::Config(e) => write!(f, "{e}"),
            CliError::Open(e) => write!(f, "{e}"),
            CliError::Usage(s) | CliError::Io(s) => f.write_str(s),
            CliError::Ambiguous { text, matches } => {
                let shorts: Vec<&str> = matches.iter().map(Oid::short).collect();
                write!(
                    f,
                    "{text} matches more than one object: {}",
                    shorts.join(", ")
                )
            }
            CliError::Cancelled => f.write_str("cancelled"),
        }
    }
}

impl std::error::Error for CliError {}

impl From<UseCaseError> for CliError {
    fn from(e: UseCaseError) -> CliError {
        CliError::UseCase(e)
    }
}
impl From<Refusal> for CliError {
    fn from(r: Refusal) -> CliError {
        CliError::UseCase(UseCaseError::Refused(r))
    }
}
impl From<ConfigError> for CliError {
    fn from(e: ConfigError) -> CliError {
        CliError::Config(e)
    }
}
impl From<OpenError> for CliError {
    fn from(e: OpenError) -> CliError {
        CliError::Open(e)
    }
}
impl From<StoreError> for CliError {
    fn from(e: StoreError) -> CliError {
        CliError::UseCase(UseCaseError::Store(e))
    }
}
impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> CliError {
        CliError::Io(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dam_application::EditorError;

    fn oid(byte: u8) -> Oid {
        Oid::generate(&mut |x: &mut [u8]| x.fill(byte))
    }

    #[test]
    fn a_refusal_names_the_rule_it_broke_and_another_failure_names_none() {
        let refused = CliError::UseCase(UseCaseError::Refused(Refusal::NothingToCommit));
        assert_eq!(refused.document()["error"]["rule"], "nothing_to_commit");
        let failed = CliError::Io("disk".into());
        assert_eq!(
            failed.document()["error"]["rule"],
            serde_json::Value::Null,
            "only a refusal breaks a rule"
        );
    }

    #[test]
    fn a_blocked_completion_names_the_task_and_then_its_blockers() {
        let error = CliError::UseCase(UseCaseError::Refused(Refusal::Blocked {
            oid: oid(1),
            blockers: vec![Blocker::OpenChild(oid(2)), Blocker::OpenDependency(oid(3))],
        }));
        let document = error.document();
        assert_eq!(document["error"]["kind"], "refused");
        assert_eq!(
            document["error"]["oids"],
            serde_json::json!([oid(1).to_string(), oid(2).to_string(), oid(3).to_string()])
        );
    }

    #[test]
    fn the_message_is_the_sentence_the_human_form_prints() {
        let error = CliError::UseCase(UseCaseError::Refused(Refusal::NotCompleted(oid(4))));
        assert_eq!(error.document()["error"]["message"], error.to_string());
    }

    #[test]
    fn an_ambiguous_prefix_is_a_usage_failure_listing_what_it_matched() {
        let error = CliError::Ambiguous {
            text: "abab".into(),
            matches: vec![oid(5), oid(6)],
        };
        assert_eq!(error.document()["error"]["kind"], "usage");
        assert_eq!(
            error.document()["error"]["oids"],
            serde_json::json!([oid(5).to_string(), oid(6).to_string()])
        );
    }

    #[test]
    fn a_failure_naming_no_object_carries_an_empty_list() {
        let error = CliError::UseCase(UseCaseError::Editor(EditorError("no editor".into())));
        assert_eq!(error.document()["error"]["oids"], serde_json::json!([]));
        assert_eq!(error.document()["error"]["rule"], serde_json::Value::Null);
    }

    #[test]
    fn a_config_file_that_cannot_be_read_is_a_store_failure_and_a_bad_one_a_parse_failure() {
        assert_eq!(
            CliError::Config(ConfigError::Io("x".into())).kind(),
            "store"
        );
        assert_eq!(
            CliError::Config(ConfigError::Syntax("x".into())).kind(),
            "parse"
        );
    }
}
