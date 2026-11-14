use dam_application::{Refusal, UseCaseError};

use super::CliError;

impl CliError {
    pub(crate) fn given_prefix(&self) -> Option<&str> {
        match self {
            CliError::UseCase(UseCaseError::Refused(
                Refusal::NoSuchObject(text)
                | Refusal::NoWorkingObject(text)
                | Refusal::AmbiguousPrefix { text, .. },
            ))
            | CliError::UseCase(UseCaseError::NotAnOid(text)) => Some(text),
            _ => None,
        }
    }
}
