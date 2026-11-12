use dam_domain::Oid;

use super::EditFields;
use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;
use crate::use_cases::load::no_such_object;

impl EditFields {
    fn named(&self) -> impl Iterator<Item = &Oid> {
        self.add_depends
            .iter()
            .chain(&self.remove_depends)
            .chain(self.attach.iter().flatten())
    }
}

impl Dam<'_> {
    pub(super) fn refuse_what_a_token_may_not_edit(
        &self,
        fields: &EditFields,
    ) -> Result<(), UseCaseError> {
        if self.access.token().is_none() {
            return Ok(());
        }
        if fields.focus.is_some() {
            return Err(Refusal::NotOpenToTokens("edit --focus").into());
        }
        for named in fields.named() {
            if self.hides(named)? {
                return Err(no_such_object(named).into());
            }
        }
        Ok(())
    }
}
