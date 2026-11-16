use dam_application::{ConfigError, HelperError, Refusal, UseCaseError};

use super::{CliError, KIND_OF_AN_EDITOR_FAILURE_NO_DOCUMENT_CARRIES, KIND_USAGE};

impl CliError {
    pub(crate) fn kind(&self) -> &'static str {
        match self {
            CliError::UseCase(UseCaseError::Refused(Refusal::AmbiguousPrefix { .. }))
            | CliError::UseCase(UseCaseError::NotAnOid(_)) => KIND_USAGE,
            CliError::UseCase(UseCaseError::Refused(_)) => "refused",
            CliError::UseCase(UseCaseError::Store(_))
            | CliError::Open(_)
            | CliError::Io(_)
            | CliError::Config(
                ConfigError::Unreadable { .. }
                | ConfigError::Unwritable { .. }
                | ConfigError::ReadOnly { .. },
            ) => "store",
            CliError::UseCase(UseCaseError::PushStopped {
                by: HelperError::Cancelled,
                ..
            }) => "cancelled",
            CliError::UseCase(UseCaseError::Helper(_) | UseCaseError::PushStopped { .. }) => {
                "helper"
            }
            CliError::UseCase(UseCaseError::Credential(_)) => "credential",
            CliError::UseCase(UseCaseError::Filter(_)) => "filter",
            CliError::UseCase(UseCaseError::Schedule(_)) => "scheduler",
            CliError::Editor(_) => KIND_OF_AN_EDITOR_FAILURE_NO_DOCUMENT_CARRIES,
            CliError::UseCase(UseCaseError::Parse(_))
            | CliError::UseCase(UseCaseError::Query(_))
            | CliError::Config(_) => "parse",
            CliError::Usage(_)
            | CliError::CommandLine(_)
            | CliError::UseCase(UseCaseError::Select(_)) => KIND_USAGE,
            CliError::Cancelled => "cancelled",
            CliError::Internal(_) | CliError::Reported => "internal",
        }
    }
}
