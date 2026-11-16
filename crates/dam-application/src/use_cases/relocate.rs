use dam_domain::{Oid, Path, nearest_parents};

use crate::errors::{Refusal, Retry, UseCaseError};
use crate::service::Dam;
use crate::use_cases::atomically::atomically;
use crate::use_cases::load::load;
use crate::use_cases::placement::Placement;
use crate::use_cases::subtree::{is_held_by, move_into};

impl Dam<'_> {
    pub fn relocate(
        &self,
        oid: &Oid,
        to: Option<&Path>,
        parent: Option<&Oid>,
    ) -> Result<(), UseCaseError> {
        if self.access.token().is_some() {
            return Err(Refusal::NotOpenToTokens("mv").into());
        }
        atomically(self.store, || self.relocate_in_place(oid, to, parent))
    }

    pub(crate) fn relocate_in_place(
        &self,
        oid: &Oid,
        to: Option<&Path>,
        parent: Option<&Oid>,
    ) -> Result<(), UseCaseError> {
        load(self.store, oid)?;
        let asked = Placement {
            path: to,
            parent,
            writing: None,
        };
        let (to, parent) = self.placement(asked, || Retry::Move(oid.clone()))?;
        let parents = match &parent {
            Some(parent) => vec![parent.clone()],
            None => nearest_parents(&to, &self.store.all()?),
        };
        for parent in &parents {
            if is_held_by(self.store, parent, oid)? {
                return Err(Refusal::MoveInsideItself(oid.clone()).into());
            }
        }
        let parent = match <[Oid; 1]>::try_from(parents) {
            Ok([parent]) => Some(parent),
            Err(_) => None,
        };
        move_into(self.store, oid, &to, parent)
    }
}

#[cfg(test)]
mod tests;
