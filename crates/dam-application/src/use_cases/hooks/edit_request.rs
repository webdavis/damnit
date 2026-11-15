use dam_domain::Oid;

use crate::errors::UseCaseError;
use crate::ports::{EditRequest, FunctionFailure};
use crate::service::Dam;

impl Dam<'_> {
    pub(super) fn hook_edit(
        &self,
        task: &Oid,
        request: &EditRequest,
    ) -> Result<Option<FunctionFailure>, UseCaseError> {
        let fields = match self.edit_requests.fields(task, request) {
            Ok(fields) => fields,
            Err(why) => return Ok(Some(request.failed(why))),
        };
        match self.edit_in_place(task, fields) {
            Ok(_) => Ok(None),
            Err(UseCaseError::Refused(refusal)) => {
                Ok(Some(request.failed(self.edit_requests.refused(&refusal))))
            }
            Err(UseCaseError::Parse(why)) => Ok(Some(request.failed(why))),
            Err(other) => Err(other),
        }
    }
}
