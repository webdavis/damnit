use std::fmt;

use dam_adapters::OpenError;
use dam_application::{ConfigError, Refusal, StoreError, UseCaseError};

use crate::editor::EditorError;

mod refusal_oids;

use refusal_oids::refusal_oids;

#[derive(Debug)]
pub(crate) enum CliError {
    UseCase(UseCaseError),
    Editor(EditorError),
    Config(ConfigError),
    Open(OpenError),
    Usage(String),
    Cancelled,
    Io(String),
    Internal(String),
}

const EXIT_FAILED: i32 = 1;
const EXIT_USAGE_AS_CLAP_EXITS: i32 = 2;
const EXIT_CANCELLED: i32 = 3;
const EXIT_REFUSED_BY_A_RULE: i32 = 4;

const KIND_OF_AN_EDITOR_FAILURE_NO_DOCUMENT_CARRIES: &str = "editor";

impl CliError {
    pub(crate) fn exit_code(&self) -> i32 {
        match self {
            CliError::Usage(_)
            | CliError::UseCase(UseCaseError::Refused(Refusal::AmbiguousPrefix { .. }))
            | CliError::UseCase(UseCaseError::NotAnOid(_))
            | CliError::UseCase(UseCaseError::Query(_)) => EXIT_USAGE_AS_CLAP_EXITS,
            CliError::Cancelled => EXIT_CANCELLED,
            CliError::UseCase(UseCaseError::Refused(_)) => EXIT_REFUSED_BY_A_RULE,
            _ => EXIT_FAILED,
        }
    }

    pub(crate) fn kind(&self) -> &'static str {
        match self {
            CliError::UseCase(UseCaseError::Refused(Refusal::AmbiguousPrefix { .. }))
            | CliError::UseCase(UseCaseError::NotAnOid(_)) => "usage",
            CliError::UseCase(UseCaseError::Refused(_)) => "refused",
            CliError::UseCase(UseCaseError::Store(_))
            | CliError::Open(_)
            | CliError::Io(_)
            | CliError::Config(ConfigError::Io(_)) => "store",
            CliError::UseCase(UseCaseError::Helper(_)) => "helper",
            CliError::UseCase(UseCaseError::Credential(_)) => "credential",
            CliError::Editor(_) => KIND_OF_AN_EDITOR_FAILURE_NO_DOCUMENT_CARRIES,
            CliError::UseCase(UseCaseError::Parse(_))
            | CliError::UseCase(UseCaseError::Query(_))
            | CliError::Config(_) => "parse",
            CliError::Usage(_) => "usage",
            CliError::Cancelled => "cancelled",
            CliError::Internal(_) => "internal",
        }
    }

    pub(crate) fn oids_in_the_order_the_message_names_them(&self) -> Vec<String> {
        match self {
            CliError::UseCase(UseCaseError::Refused(refusal)) => refusal_oids(refusal),
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
            CliError::Editor(e) => write!(f, "editor: {}", e.0),
            CliError::Config(e) => write!(f, "{e}"),
            CliError::Open(e) => write!(f, "{e}"),
            CliError::Usage(text) | CliError::Io(text) | CliError::Internal(text) => {
                f.write_str(text)
            }
            CliError::Cancelled => f.write_str("cancelled"),
        }
    }
}

impl std::error::Error for CliError {}

impl From<EditorError> for CliError {
    fn from(e: EditorError) -> CliError {
        CliError::Editor(e)
    }
}
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
