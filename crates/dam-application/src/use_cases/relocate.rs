use dam_domain::{Oid, Path, nearest_parents};

use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;
use crate::use_cases::atomically::atomically;
use crate::use_cases::load::load;
use crate::use_cases::subtree::{is_held_by, move_into};

impl Dam<'_> {
    pub fn relocate(&self, oid: &Oid, to: &Path) -> Result<(), UseCaseError> {
        atomically(self.store, || {
            load(self.store, oid)?;
            let parents = nearest_parents(to, &self.store.all()?);
            for parent in &parents {
                if is_held_by(self.store, parent, oid)? {
                    return Err(Refusal::MoveInsideItself(oid.clone()).into());
                }
            }
            let parent = match <[Oid; 1]>::try_from(parents) {
                Ok([parent]) => Some(parent),
                Err(_) => None,
            };
            move_into(self.store, oid, to, parent)
        })
    }
}

#[cfg(test)]
mod tests;
