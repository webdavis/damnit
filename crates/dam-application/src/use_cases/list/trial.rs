use dam_domain::{Command, Object, Oid, Path};

use crate::use_cases::selected::Defaults;

use crate::errors::{FilterError, UseCaseError};
use crate::ports::{FunctionFailure, FunctionFault};
use crate::service::Dam;

#[derive(Debug, PartialEq, Eq)]
pub enum FilterTrial {
    Passed {
        ran_on: usize,
        kept: usize,
    },
    Failed {
        failure: FunctionFailure,
        object: Option<Oid>,
    },
}

impl Dam<'_> {
    pub fn try_filter(&self, name: &str) -> Result<FilterTrial, UseCaseError> {
        let ran_on = self.seen_within(&Path::root())?.len();
        let (config, narrowed) =
            self.listed_by_table(name, Defaults::Of(Command::List), (&Path::root(), &[]), &[])?;
        match self.kept(&config, narrowed.clone()) {
            Ok(kept) => Ok(FilterTrial::Passed {
                ran_on,
                kept: kept.len(),
            }),
            Err(UseCaseError::Filter(FilterError::Failed { failure, .. })) => {
                let object = match failure.fault {
                    FunctionFault::RanOver => None,
                    _ => self.first_failing(name, narrowed),
                };
                Ok(FilterTrial::Failed { failure, object })
            }
            Err(error) => Err(error),
        }
    }

    fn first_failing(&self, name: &str, narrowed: Vec<Object>) -> Option<Oid> {
        narrowed.into_iter().find_map(|object| {
            let oid = object.oid().clone();
            self.kept_by_function(name, vec![object]).err().map(|_| oid)
        })
    }
}
