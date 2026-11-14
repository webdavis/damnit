use dam_domain::{Object, Oid, Permission};

use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;
use crate::use_cases::atomically::atomically;
use crate::use_cases::items::land::Landed;
use crate::use_cases::load::no_such_object;
use crate::use_cases::subtree::refuse_losing_a_held_folder;

pub use fields::{EditFields, apply};

impl Dam<'_> {
    pub fn edit(&self, oid: &Oid, fields: EditFields) -> Result<Landed, UseCaseError> {
        atomically(self.store, || {
            let current = self.load_permitted(oid, Permission::change)?;
            self.refuse_what_a_token_may_not_edit(&fields)?;
            let mut next = apply(&current, &fields)?;
            if let Some(day) = fields.focus {
                self.mark_focus(&mut next, day)?;
            }
            self.config
                .categories
                .check(&next.base().labels)
                .map_err(Refusal::Labels)?;
            if self.access.token().is_none()
                && let Some(path) = self.cycle_through(oid, &fields.add_depends)?
            {
                return Err(Refusal::Cycle {
                    oid: oid.clone(),
                    path,
                }
                .into());
            }
            if let Some(Some(event)) = &fields.attach
                && !matches!(self.store.get(event)?, Some(Object::Event(_)))
            {
                return Err(no_such_object(event).into());
            }
            refuse_losing_a_held_folder(self.store, &current, &next)?;
            self.put_or_send(Permission::change(current.kind()), Some(&current), next)
        })
    }
}

mod depends_cycle;
mod fields;
mod focus;
mod token_limits;

#[cfg(test)]
mod tests;
