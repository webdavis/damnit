use dam_domain::{Oid, Path};

use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;
use crate::use_cases::atomically::atomically;
use crate::use_cases::load::load;
use crate::use_cases::subtree::move_into;

impl Dam<'_> {
    pub fn relocate(&self, oid: &Oid, to: &Path) -> Result<(), UseCaseError> {
        atomically(self.store, || {
            let object = load(self.store, oid)?;
            if object
                .base()
                .owned_folder()
                .is_some_and(|folder| to.is_within(&folder))
            {
                return Err(Refusal::MoveInsideItself(oid.clone()).into());
            }
            move_into(self.store, oid, to)
        })
    }
}

#[cfg(test)]
mod tests;
