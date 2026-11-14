use std::fmt;

use dam_application::{FilterError, FilterFault};

pub(super) struct FilterText<'a>(pub(super) &'a FilterError);

impl fmt::Display for FilterText<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            FilterError::Withheld { filter } => write!(
                f,
                "filter {filter} failed; the owner sees why by running the same command"
            ),
            FilterError::Failed { filter, failure } => {
                write!(f, "filter {filter} failed at {}", failure.file.display())?;
                if let Some(line) = failure.line {
                    write!(f, ":{line}")?;
                }
                match &failure.fault {
                    FilterFault::Raised { message } => write!(f, ": {message}"),
                    FilterFault::RanOver => {
                        f.write_str(": match ran past the time limit for one listing")
                    }
                    FilterFault::Returned { returned } => {
                        write!(f, ": match returned {returned}, not true, false or nil")
                    }
                }
            }
        }
    }
}
