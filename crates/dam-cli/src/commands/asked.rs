use dam_application::{SelectError, UseCaseError};
use dam_domain::{Filter, FilterTable};

use crate::error::{CliError, Fault};

pub(crate) struct Asked<'a> {
    pub(crate) verb: String,
    pub(crate) typed: &'a FilterTable,
    pub(crate) sort: &'a [String],
}

impl Asked<'_> {
    pub(crate) fn filter(&self) -> Result<Filter, CliError> {
        Filter::read(self.typed).map_err(|e| self.fault(SelectError::Value(e)))
    }

    pub(crate) fn fault(&self, error: SelectError) -> CliError {
        CliError::Selection(Box::new(Fault {
            error,
            typed: self.typed.clone(),
            sort: self.sort.to_vec(),
            verb: self.verb.clone(),
        }))
    }

    pub(crate) fn refused(&self, error: UseCaseError) -> CliError {
        match error {
            UseCaseError::Select(error) => self.fault(error),
            e => e.into(),
        }
    }
}
