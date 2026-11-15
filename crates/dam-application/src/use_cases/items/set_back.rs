use dam_domain::{Author, ItemHash, Oid};

use super::accept::state;
use crate::errors::UseCaseError;
use crate::ports::ObjectState;
use crate::service::Dam;

impl Dam<'_> {
    pub(super) fn set_back_what_followed(&self, item: &ItemHash) -> Result<Vec<Oid>, UseCaseError> {
        let follow_ons = self.store.follow_ons(item)?;
        if follow_ons.is_empty() {
            return Ok(Vec::new());
        }
        let mut kept = Vec::new();
        for follow_on in follow_ons {
            let oid = follow_on.after.working.oid().clone();
            if self.state_of(&oid)?.as_ref() == Some(&follow_on.after) {
                self.put_back(&oid, follow_on.before)?;
            } else {
                kept.push(oid);
            }
        }
        self.store.forget_follow_ons(item)?;
        Ok(kept)
    }

    fn state_of(&self, oid: &Oid) -> Result<Option<ObjectState>, UseCaseError> {
        let staged = self.store.staged()?;
        Ok(self.store.get(oid)?.map(|working| state(working, &staged)))
    }

    fn put_back(&self, oid: &Oid, before: ObjectState) -> Result<(), UseCaseError> {
        let sent: Vec<_> = self
            .store
            .staged()?
            .into_iter()
            .filter(|s| s.author != Author::Owner && s.change.oid() == oid)
            .collect();
        self.store.unstage(oid)?;
        for staged in before.owners_staged.into_iter().chain(sent) {
            self.store.put_staged(staged)?;
        }
        self.store.put(&before.working)?;
        Ok(())
    }
}
