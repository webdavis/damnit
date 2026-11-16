use dam_adapters::Interrupt;
use dam_application::{HelperError, UseCaseError};

use crate::error::CliError;

pub(crate) fn cancelled_when_interrupted_whatever_the_work_reported(
    e: CliError,
    interrupt: &Interrupt,
) -> CliError {
    match e {
        _ if !interrupt.requested() => e,
        CliError::UseCase(UseCaseError::PushStopped {
            kept, unanswered, ..
        }) => CliError::UseCase(UseCaseError::PushStopped {
            by: HelperError::Cancelled,
            kept,
            unanswered,
        }),
        _ => CliError::Cancelled,
    }
}

#[cfg(test)]
mod tests;
