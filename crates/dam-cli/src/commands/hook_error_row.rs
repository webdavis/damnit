use dam_application::FunctionFailure;
use serde::Serialize;

use crate::error::FailureText;

#[derive(Serialize)]
pub(crate) struct HookErrorRow {
    file: String,
    line: Option<u32>,
    message: String,
}

impl HookErrorRow {
    pub(crate) fn of(failure: &FunctionFailure) -> HookErrorRow {
        HookErrorRow {
            file: failure.file.display().to_string(),
            line: failure.line,
            message: FailureText(failure).to_string(),
        }
    }

    pub(crate) fn line(&self) -> String {
        let at = match self.line {
            Some(line) => format!("{}:{line}", self.file),
            None => self.file.clone(),
        };
        format!("first error at {at}: {}", self.message)
    }
}
