use dam_domain::Object;

use super::{EditFields, apply};
use crate::errors::UseCaseError;
use crate::service::Dam;
use crate::use_cases::load::no_such_object;

impl Dam<'_> {
    pub(crate) fn set_on(
        &self,
        current: &Object,
        fields: &EditFields,
    ) -> Result<Object, UseCaseError> {
        self.refuse_what_a_token_may_not_edit(fields)?;
        let mut next = apply(current, fields)?;
        if let Some(day) = fields.focus {
            self.mark_focus(&mut next, day)?;
        }
        if let Some(Some(event)) = &fields.attach
            && !matches!(self.store.get(event)?, Some(Object::Event(_)))
        {
            return Err(no_such_object(event).into());
        }
        Ok(next)
    }
}
