use dam_adapters::OpenError;
use dam_application::{ConfigError, Refusal, StoreError, UseCaseError};

use super::CliError;
use crate::editor::EditorError;

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
