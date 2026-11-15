use dam_application::{Refusal, UseCaseError};

use super::CliError;

impl CliError {
    pub(crate) fn crontab_line(&self) -> Option<&str> {
        match self {
            CliError::UseCase(UseCaseError::Refused(Refusal::NoScheduler { crontab_line })) => {
                Some(crontab_line)
            }
            _ => None,
        }
    }
}
