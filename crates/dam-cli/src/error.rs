use std::fmt;

use dam_adapters::{ConfigError, OpenError};
use dam_application::{Refusal, StoreError, UseCaseError};
use dam_domain::Oid;

mod refusal_oids;

use refusal_oids::refusal_oids;

#[derive(Debug)]
pub(crate) enum CliError {
    UseCase(UseCaseError),
    Config(ConfigError),
    Open(OpenError),
    Usage(String),
    Ambiguous { text: String, matches: Vec<Oid> },
    Cancelled,
    Io(String),
}

const EXIT_FAILED: i32 = 1;
const EXIT_USAGE_AS_CLAP_EXITS: i32 = 2;
const EXIT_CANCELLED: i32 = 3;
const EXIT_REFUSED_BY_A_RULE: i32 = 4;

const KIND_OF_AN_EDITOR_FAILURE_NO_DOCUMENT_CARRIES: &str = "editor";

impl CliError {
    pub(crate) fn exit_code(&self) -> i32 {
        match self {
            CliError::Usage(_) | CliError::Ambiguous { .. } => EXIT_USAGE_AS_CLAP_EXITS,
            CliError::Cancelled => EXIT_CANCELLED,
            CliError::UseCase(UseCaseError::Refused(_)) => EXIT_REFUSED_BY_A_RULE,
            _ => EXIT_FAILED,
        }
    }

    pub(crate) fn kind(&self) -> &'static str {
        match self {
            CliError::UseCase(UseCaseError::Refused(_)) => "refused",
            CliError::UseCase(UseCaseError::Store(_))
            | CliError::Open(_)
            | CliError::Io(_)
            | CliError::Config(ConfigError::Io(_)) => "store",
            CliError::UseCase(UseCaseError::Helper(_)) => "helper",
            CliError::UseCase(UseCaseError::Credential(_)) => "credential",
            CliError::UseCase(UseCaseError::Editor(_)) => {
                KIND_OF_AN_EDITOR_FAILURE_NO_DOCUMENT_CARRIES
            }
            CliError::UseCase(UseCaseError::Parse(_)) | CliError::Config(_) => "parse",
            CliError::Usage(_) | CliError::Ambiguous { .. } => "usage",
            CliError::Cancelled => "cancelled",
        }
    }

    pub(crate) fn oids_in_the_order_the_message_names_them(&self) -> Vec<String> {
        match self {
            CliError::UseCase(UseCaseError::Refused(refusal)) => refusal_oids(refusal),
            CliError::Ambiguous { matches, .. } => matches.iter().map(Oid::to_string).collect(),
            _ => Vec::new(),
        }
    }

    pub(crate) fn rule(&self) -> Option<&'static str> {
        match self {
            CliError::UseCase(UseCaseError::Refused(refusal)) => Some(refusal.name()),
            _ => None,
        }
    }

    pub(crate) fn document(&self) -> serde_json::Value {
        serde_json::json!({
            "error": {
                "kind": self.kind(),
                "rule": self.rule(),
                "message": self.to_string(),
                "oids": self.oids_in_the_order_the_message_names_them(),
            }
        })
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
mod tests;
