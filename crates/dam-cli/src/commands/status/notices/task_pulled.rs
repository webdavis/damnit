use dam_application::{FunctionFailure, RemoteName};
use serde::Serialize;

use crate::error::FaultText;

#[derive(Serialize)]
pub(in crate::commands::status) struct TaskPulledFailedRow<'a> {
    remote: &'a str,
    failed: usize,
    not_reached: usize,
    error: Option<HookErrorRow>,
}

#[derive(Serialize)]
struct HookErrorRow {
    file: String,
    line: Option<u32>,
    message: String,
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
            error: error.map(|failure| HookErrorRow {
                file: failure.file.display().to_string(),
                line: failure.line,
                message: FaultText(&failure.fault).to_string(),
            }),
        }
    }

    pub(super) fn line(&self) -> String {
        let counts = format!(
            "task_pulled on the last pull from {}: {} failed, {} not reached",
            self.remote, self.failed, self.not_reached
        );
        let Some(error) = &self.error else {
            return counts;
        };
        let at = match error.line {
            Some(line) => format!("{}:{line}", error.file),
            None => error.file.clone(),
        };
        format!("{counts}; first error at {at}: {}", error.message)
    }
}
