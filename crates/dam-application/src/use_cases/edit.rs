use dam_domain::{Oid, Permission};

use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;
use crate::use_cases::atomically::atomically;
use crate::use_cases::items::land::Landed;
use crate::use_cases::subtree::refuse_losing_a_held_folder;

pub use fields::{EditFields, apply};

impl Dam<'_> {
    pub fn edit(&self, oid: &Oid, fields: EditFields) -> Result<Landed, UseCaseError> {
        atomically(self.store, || {
            self.entering(&[], || self.edit_in_place(oid, fields))
        })
    }

    pub(crate) fn edit_in_place(
        &self,
        oid: &Oid,
        fields: EditFields,
    ) -> Result<Landed, UseCaseError> {
        let current = self.load_permitted(oid, Permission::change)?;
        let next = self.set_on(&current, &fields)?;
        self.config
            .categories_at(&next.base().path)
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
        self.unless_it_names_the_unseen(refuse_losing_a_held_folder(self.store, &current, &next))?;
        self.put_or_send(Permission::change(current.kind()), Some(&current), next)
    }
}

mod depends_cycle;
mod fields;
mod focus;
mod set_on;
mod token_limits;

#[cfg(test)]
mod tests;
