use std::fmt;

use dam_adapters::{ConfigError, OpenError};
use dam_application::{StoreError, UseCaseError};
use dam_domain::Oid;

#[derive(Debug)]
pub enum CliError {
    UseCase(UseCaseError),
    Config(ConfigError),
    Open(OpenError),
    Usage(String),
    /// Constructed by `oids::resolve_oid` when a prefix or path names more than one object.
    Ambiguous {
        text: String,
        matches: Vec<Oid>,
    },
    /// A prompt hit EOF or an unscripted answer; `TerminalPrompt` and `ScriptedPrompt` both return it.
    Cancelled,
    Io(String),
}

impl CliError {
    pub fn exit_code(&self) -> i32 {
        match self {
            CliError::UseCase(UseCaseError::Refused(_)) => 2,
            CliError::Cancelled => 3,
            _ => 1,
        }
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
