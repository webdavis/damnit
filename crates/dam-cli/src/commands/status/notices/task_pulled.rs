use dam_application::{FunctionFailure, RemoteName};
use serde::Serialize;

use crate::commands::hook_error_row::HookErrorRow;

#[derive(Serialize)]
pub(in crate::commands::status) struct TaskPulledFailedRow<'a> {
    remote: &'a str,
    failed: usize,
    not_reached: usize,
    error: Option<HookErrorRow>,
}

impl<'a> TaskPulledFailedRow<'a> {
    pub(super) fn new(
        remote: &'a RemoteName,
        failed: usize,
        not_reached: usize,
        error: Option<&FunctionFailure>,
    ) -> TaskPulledFailedRow<'a> {
        TaskPulledFailedRow {
            remote: &remote.0,
            failed,
            not_reached,
            error: error.map(HookErrorRow::of),
        }
    }

    pub(super) fn line(&self) -> String {
        let counts = format!(
            "task_pulled on the last pull from {}: {} failed, {} not reached",
            self.remote, self.failed, self.not_reached
        );
        match &self.error {
            Some(error) => format!("{counts}; {}", error.line()),
            None => counts,
        }
    }
}
