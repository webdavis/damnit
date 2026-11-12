use dam_domain::{Item, ItemHash, Outcome};

use super::refusal::ItemRefusal;
use crate::errors::UseCaseError;
use crate::service::Dam;
use crate::use_cases::atomically::atomically;

impl Dam<'_> {
    pub fn dismiss(&self, hash: &ItemHash) -> Result<Item, UseCaseError> {
        atomically(self.store, || {
            let mut item = self.waiting_item(hash)?;
            let now = self.clock.now();
            item.outcome = match self.access.token() {
                None => Outcome::Declined(now),
                Some(name) if name == item.author.name => Outcome::Withdrawn(now),
                Some(_) => return Err(ItemRefusal::NotYours(hash.clone()).into()),
            };
            self.store.settle(hash, item.outcome)?;
            Ok(item)
        })
    }
}

#[cfg(test)]
mod tests;
