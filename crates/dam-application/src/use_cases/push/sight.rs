use dam_domain::{Object, Oid, Permission, Subject};

use crate::errors::UseCaseError;
use crate::service::Dam;

pub(super) const HELD_BY_WHAT_YOU_CANNOT_SEE: &str = "held until a task you cannot see is settled";

impl Dam<'_> {
    pub(super) fn any_hidden(&self, oids: &[Oid]) -> Result<bool, UseCaseError> {
        for oid in oids {
            let working = self.store.get(oid)?;
            let committed = self.store.committed(oid)?;
            let hidden_in =
                |object: &Option<Object>| object.as_ref().is_some_and(|o| !self.access.sees(o));
            if (working.is_none() && committed.is_none())
                || hidden_in(&working)
                || hidden_in(&committed)
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub(super) fn reads(&self, oid: &Oid) -> Result<bool, UseCaseError> {
        Ok(!self.any_hidden(std::slice::from_ref(oid))?)
    }

    pub(super) fn reads_everything(&self) -> bool {
        let whole = |permission| self.access.permits(permission, &[Subject::Whole]);
        whole(Permission::TaskRead) && whole(Permission::EventRead)
    }
}
