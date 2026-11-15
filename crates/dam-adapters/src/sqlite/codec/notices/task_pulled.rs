use std::path::PathBuf;

use dam_application::{FunctionFailure, FunctionFault, Notice, RemoteName};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(super) struct TaskPulledFailedRow {
    remote: String,
    failed: usize,
    not_reached: usize,
    error: Option<ErrorRow>,
}

#[derive(Serialize, Deserialize)]
struct ErrorRow {
    file: PathBuf,
    line: Option<u32>,
    fault: FaultRow,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum FaultRow {
    Raised { message: String },
    RanOver,
    Returned { returned: String },
    Requested { why: String },
}

impl TaskPulledFailedRow {
    pub(super) fn new(
        remote: &RemoteName,
        failed: usize,
        not_reached: usize,
        error: Option<&FunctionFailure>,
    ) -> TaskPulledFailedRow {
        TaskPulledFailedRow {
            remote: remote.0.clone(),
            failed,
            not_reached,
            error: error.map(|e| ErrorRow {
                file: e.file.clone(),
                line: e.line,
                fault: match e.fault.clone() {
                    FunctionFault::Raised { message } => FaultRow::Raised { message },
                    FunctionFault::RanOver => FaultRow::RanOver,
                    FunctionFault::Returned { returned } => FaultRow::Returned { returned },
                    FunctionFault::Requested { why } => FaultRow::Requested { why },
                },
            }),
        }
    }

    pub(super) fn notice(self) -> Notice {
        Notice::TaskPulledFailed {
            remote: RemoteName(self.remote),
            failed: self.failed,
            not_reached: self.not_reached,
            error: self.error.map(|e| FunctionFailure {
                file: e.file,
                line: e.line,
                fault: match e.fault {
                    FaultRow::Raised { message } => FunctionFault::Raised { message },
                    FaultRow::RanOver => FunctionFault::RanOver,
                    FaultRow::Returned { returned } => FunctionFault::Returned { returned },
                    FaultRow::Requested { why } => FunctionFault::Requested { why },
                },
            }),
        }
    }
}
