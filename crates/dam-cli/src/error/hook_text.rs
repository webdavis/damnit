use std::fmt;

use dam_application::{FunctionFault, HookFailure};

pub(crate) struct HookText<'a>(pub(crate) &'a HookFailure);

impl fmt::Display for HookText<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let HookFailure { event, failure } = self.0;
        write!(
            f,
            "hook {} failed at {}",
            event.as_str(),
            failure.file.display()
        )?;
        if let Some(line) = failure.line {
            write!(f, ":{line}")?;
        }
        match &failure.fault {
            FunctionFault::Raised { message } => write!(f, ": {message}"),
            FunctionFault::RanOver => f.write_str(": it ran past the time limit for one hook call"),
            FunctionFault::Returned { returned } => {
                write!(f, ": it returned {returned}, not nil or a table")
            }
            FunctionFault::Requested { why } => write!(f, ": its edit request failed: {why}"),
        }
    }
}
