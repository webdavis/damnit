use dam_domain::{Change, Object, Oid, Permission, Subject};

use crate::errors::UseCaseError;
use crate::service::Dam;

pub(crate) const HELD_BY_WHAT_YOU_CANNOT_SEE: &str = "held until a task you cannot see is settled";

impl Dam<'_> {
    pub(crate) fn any_hidden(&self, oids: &[Oid]) -> Result<bool, UseCaseError> {
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

    pub(crate) fn reads(&self, oid: &Oid) -> Result<bool, UseCaseError> {
        Ok(!self.any_hidden(std::slice::from_ref(oid))?)
    }

    pub(crate) fn reads_everything(&self) -> bool {
        let whole = |permission| self.access.permits(permission, &[Subject::Whole]);
        whole(Permission::TaskRead) && whole(Permission::EventRead)
    }

    pub(crate) fn reads_change(&self, change: &Change) -> Result<bool, UseCaseError> {
        let every_state_seen = change
            .before()
            .into_iter()
            .chain(change.after())
            .all(|object| self.access.sees(object));
        Ok(every_state_seen && !self.hidden_now(change.oid())?)
    }

    fn hidden_now(&self, oid: &Oid) -> Result<bool, UseCaseError> {
        let unseen = |object: Option<Object>| object.is_some_and(|o| !self.access.sees(&o));
        Ok(unseen(self.store.get(oid)?) || unseen(self.store.committed(oid)?))
    }
}
