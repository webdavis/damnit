use dam_application::{Refusal, UseCaseError};

use super::CliError;

impl CliError {
    pub(crate) fn given_key(&self) -> Option<&str> {
        match self {
            CliError::UseCase(UseCaseError::Refused(Refusal::NoSuchSetting(key))) => Some(key),
            _ => None,
        }
    }
}
