use dam_adapters::Interrupt;
use dam_application::{HelperError, UseCaseError};

use crate::error::CliError;
use crate::output::Format;

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

pub(crate) fn warn_a_human_that_ctrl_c_will_not_cancel_cleanly(format: Format) {
    if format == Format::Human {
        eprintln!("dam: could not install the interrupt handler; Ctrl-C will not cancel cleanly");
    }
}
