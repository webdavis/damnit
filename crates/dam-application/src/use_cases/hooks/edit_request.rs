use dam_domain::Oid;

use crate::errors::UseCaseError;
use crate::ports::{EditRequest, FunctionFailure};
use crate::service::Dam;

pub(super) enum Edited {
    Nothing,
    Changed,
    Failed(FunctionFailure),
}

impl Dam<'_> {
    pub(super) fn hook_edit(
        &self,
        task: &Oid,
        request: &EditRequest,
    ) -> Result<Edited, UseCaseError> {
        let fields = match self.edit_requests.fields(task, request) {
            Ok(fields) => fields,
            Err(why) => return Ok(Edited::Failed(request.failed(why))),
        };
        let before = self.store.get(task)?;
        match self.edit_in_place(task, fields) {
            Ok(_) if self.store.get(task)? != before => Ok(Edited::Changed),
            Ok(_) => Ok(Edited::Nothing),
            Err(UseCaseError::Refused(refusal)) => Ok(Edited::Failed(
                request.failed(self.edit_requests.refused(&refusal)),
            )),
            Err(UseCaseError::Parse(why)) => Ok(Edited::Failed(request.failed(why))),
            Err(other) => Err(other),
        }
    }
}
