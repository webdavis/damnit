use std::fmt;

use dam_adapters::OpenError;
use dam_application::{ConfigError, Refusal, UseCaseError};

use crate::editor::EditorError;

mod config_text;
mod conversions;
mod edit_text;
mod item_text;
mod misread_text;
mod misuse;
mod refusal_oids;
mod refusal_text;
mod secrets_tightened_text;
mod token_fault_text;
mod token_text;
mod use_case_text;

use config_text::ConfigText;
pub(crate) use misuse::{Misuse, Problem};
use refusal_oids::refusal_oids;
pub(crate) use secrets_tightened_text::SecretsTightenedText;
pub(crate) use token_fault_text::{TokenFaultText, TokenWarningText};
pub(crate) use use_case_text::UseCaseText;

#[derive(Debug)]
pub(crate) enum CliError {
    UseCase(UseCaseError),
    Editor(EditorError),
    Config(ConfigError),
    ConfigCheck(ConfigError),
    Open(OpenError),
    Usage(String),
    CommandLine(Misuse),
    Cancelled,
    Io(String),
    Internal(String),
}

const EXIT_FAILED: i32 = 1;
const EXIT_USAGE_AS_CLAP_EXITS: i32 = 2;
const EXIT_CANCELLED: i32 = 3;
const EXIT_REFUSED_BY_A_RULE: i32 = 4;
const EXIT_TOKEN_REFUSED: i32 = 5;

const KIND_USAGE: &str = "usage";
const KIND_OF_AN_EDITOR_FAILURE_NO_DOCUMENT_CARRIES: &str = "editor";

impl CliError {
    pub(crate) fn exit_code(&self) -> i32 {
        match self {
            CliError::Usage(_)
            | CliError::CommandLine(_)
            | CliError::UseCase(UseCaseError::Refused(Refusal::AmbiguousPrefix { .. }))
            | CliError::UseCase(UseCaseError::NotAnOid(_))
            | CliError::UseCase(UseCaseError::Query(_)) => EXIT_USAGE_AS_CLAP_EXITS,
            CliError::Cancelled => EXIT_CANCELLED,
            CliError::UseCase(UseCaseError::Refused(Refusal::Token(_))) => EXIT_TOKEN_REFUSED,
            CliError::UseCase(UseCaseError::Refused(_)) => EXIT_REFUSED_BY_A_RULE,
            _ => EXIT_FAILED,
        }
    }

    pub(crate) fn kind(&self) -> &'static str {
        match self {
            CliError::UseCase(UseCaseError::Refused(Refusal::AmbiguousPrefix { .. }))
            | CliError::UseCase(UseCaseError::NotAnOid(_)) => KIND_USAGE,
            CliError::UseCase(UseCaseError::Refused(_)) => "refused",
            CliError::UseCase(UseCaseError::Store(_))
            | CliError::Open(_)
            | CliError::Io(_)
            | CliError::Config(ConfigError::Unreadable { .. })
            | CliError::ConfigCheck(ConfigError::Unreadable { .. }) => "store",
            CliError::UseCase(UseCaseError::Helper(_)) => "helper",
            CliError::UseCase(UseCaseError::Credential(_)) => "credential",
            CliError::Editor(_) => KIND_OF_AN_EDITOR_FAILURE_NO_DOCUMENT_CARRIES,
            CliError::UseCase(UseCaseError::Parse(_))
            | CliError::UseCase(UseCaseError::Query(_))
            | CliError::Config(_)
            | CliError::ConfigCheck(_) => "parse",
            CliError::Usage(_) | CliError::CommandLine(_) => KIND_USAGE,
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
        let mut error = serde_json::json!({
            "kind": self.kind(),
            "rule": self.rule(),
            "message": self.to_string(),
            "oids": self.oids_in_the_order_the_message_names_them(),
        });
        if let Some(usage) = misuse::usage_part(self) {
            error["usage"] = usage;
        }
        serde_json::json!({ "error": error })
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::UseCase(e) => write!(f, "{}", UseCaseText(e)),
            CliError::Editor(e) => write!(f, "editor: {}", e.0),
            CliError::Config(e @ ConfigError::NotLua(_)) | CliError::ConfigCheck(e) => {
                write!(f, "{}", ConfigText(e))
            }
            CliError::Config(e) => write!(f, "{}; run dam config check", ConfigText(e)),
            CliError::Open(e) => write!(f, "{e}"),
            CliError::Usage(text) | CliError::Io(text) | CliError::Internal(text) => {
                f.write_str(text)
            }
            CliError::CommandLine(misuse) => f.write_str(&misuse.message),
            CliError::Cancelled => f.write_str("cancelled"),
        }
    }
}

impl std::error::Error for CliError {}

#[cfg(test)]
mod tests;
