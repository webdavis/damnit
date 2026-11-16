use std::fmt;

use dam_application::{FilterError, FunctionFailure, FunctionFault};

pub(super) struct FilterText<'a>(pub(super) &'a FilterError);

pub(crate) struct FailureText<'a>(pub(crate) &'a FunctionFailure);

impl fmt::Display for FilterText<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            FilterError::Withheld { filter } => write!(
                f,
                "filter {filter} failed; the owner sees why by running the same command"
            ),
            FilterError::Failed { filter, failure } => {
                write!(f, "filter {filter} failed at {}", FailureText(failure))
            }
        }
    }
}

impl fmt::Display for FailureText<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let failure = self.0;
        write!(f, "{}", failure.file.display())?;
        if let Some(line) = failure.line {
            write!(f, ":{line}")?;
        }
        match &failure.fault {
            FunctionFault::Raised { message } => write!(f, ": {message}"),
            FunctionFault::RanOver => {
                f.write_str(": match ran past the time limit for one listing")
            }
            FunctionFault::Returned { returned } => {
                write!(f, ": match returned {returned}, not true, false or nil")
            }
            FunctionFault::Requested { why } => write!(f, ": {why}"),
        }
    }
}
