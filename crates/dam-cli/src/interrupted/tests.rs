use dam_adapters::Interrupt;
use dam_application::{HelperError, UseCaseError};

use super::cancelled_when_interrupted_whatever_the_work_reported as cancelled;
use crate::error::CliError;

fn cut_short_by_the_deadline() -> CliError {
    CliError::UseCase(UseCaseError::PushStopped {
        by: HelperError::Timeout {
            helper: "fake".into(),
            deadline: "30s".into(),
            said: None,
        },
        kept: 1,
        unanswered: 1,
    })
}

fn interrupted() -> Interrupt {
    let interrupt = Interrupt::default();
    interrupt.request();
    interrupt
}

#[test]
fn an_interrupted_push_exits_three_and_says_how_much_it_kept() {
    let error = cancelled(cut_short_by_the_deadline(), &interrupted());
    assert_eq!(error.exit_code(), 3);
    assert_eq!(error.to_string(), "cancelled; 1 kept, 1 unanswered");
}

#[test]
fn any_other_interrupted_work_exits_three_as_cancelled() {
    let error = cancelled(CliError::Io("disk".into()), &interrupted());
    assert_eq!(error.exit_code(), 3);
    assert_eq!(error.to_string(), "cancelled");
}

#[test]
fn work_that_was_not_interrupted_keeps_its_own_error() {
    let error = cancelled(cut_short_by_the_deadline(), &Interrupt::default());
    assert_eq!(error.exit_code(), 1);
}
