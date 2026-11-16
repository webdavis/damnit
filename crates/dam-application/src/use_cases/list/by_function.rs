use dam_domain::Object;

use super::log_by_object::LogByObject;
use crate::errors::{FilterError, UseCaseError};
use crate::ports::{FunctionFailure, Handed};
use crate::service::Dam;

impl Dam<'_> {
    pub(super) fn kept_by_function(
        &self,
        filter: &str,
        listed: Vec<Object>,
    ) -> Result<Vec<Object>, UseCaseError> {
        let worked_out = self.worked_out_over_seen(&listed)?;
        let handed: Vec<Handed> = listed
            .into_iter()
            .zip(worked_out)
            .map(|(object, worked_out)| Handed { object, worked_out })
            .collect();
        let histories = LogByObject::new(self);
        let keeps = self.filter_functions.keeps(filter, &handed, &histories);
        histories.unread()?;
        let keeps = keeps.map_err(|failure| self.failed(filter, failure))?;
        Ok(handed
            .into_iter()
            .zip(keeps)
            .filter_map(|(handed, kept)| kept.then_some(handed.object))
            .collect())
    }

    fn failed(&self, filter: &str, failure: FunctionFailure) -> UseCaseError {
        let filter = filter.to_string();
        UseCaseError::Filter(match self.access.token() {
            None => FilterError::Failed { filter, failure },
            Some(_) => FilterError::Withheld { filter },
        })
    }
}
