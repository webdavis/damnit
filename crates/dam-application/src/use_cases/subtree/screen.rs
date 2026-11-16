use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;
use crate::use_cases::load::no_such_object;

impl Dam<'_> {
    pub(crate) fn unless_it_names_the_unseen<T>(
        &self,
        result: Result<T, UseCaseError>,
    ) -> Result<T, UseCaseError> {
        let Err(UseCaseError::Refused(Refusal::ParentNotKnown {
            oid,
            unsettled,
            namesakes,
            ..
        })) = &result
        else {
            return result;
        };
        for named in unsettled.iter().chain(namesakes) {
            if self.hides(named)? {
                return Err(no_such_object(oid).into());
            }
        }
        result
    }
}
