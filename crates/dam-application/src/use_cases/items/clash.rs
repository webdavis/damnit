use dam_domain::{Change, Item, ItemHash, Oid, Outcome, clashing_fields, fields_the_owner_moved};

use super::accept::OnClash;
use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;
use crate::use_cases::atomically::atomically;
use crate::use_cases::prefix::Named;
use crate::use_cases::resolve::Side;

impl Dam<'_> {
    pub fn settle(&self, named: &Named, side: Side) -> Result<Oid, UseCaseError> {
        let oid = match named {
            Named::Item(hash) => return self.settle_item(hash, side),
            Named::Object(oid) => oid,
        };
        if self.store.conflicts()?.iter().any(|c| c.oid == *oid) {
            self.resolve(oid, side)?;
            return Ok(oid.clone());
        }
        let clashing = self.items_clashing_on(oid)?;
        match clashing.as_slice() {
            [] => Err(Refusal::NoConflict(oid.clone()).into()),
            [one] => self.settle_item(one, side),
            _ => Err(Refusal::AmbiguousPrefix {
                text: oid.short().to_string(),
                matches: clashing.iter().map(ItemHash::to_string).collect(),
            }
            .into()),
        }
    }

    fn settle_item(&self, hash: &ItemHash, side: Side) -> Result<Oid, UseCaseError> {
        atomically(self.store, || {
            let target = self.waiting_item(hash)?.target().clone();
            match side {
                Side::Theirs => {
                    self.take(hash, OnClash::TakeTheirs)?;
                }
                Side::Ours => {
                    let declined = Outcome::Declined(self.clock.now());
                    self.store.settle(hash, declined)?;
                }
            }
            Ok(target)
        })
    }

    fn items_clashing_on(&self, oid: &Oid) -> Result<Vec<ItemHash>, UseCaseError> {
        let (Some(committed), Some(working)) = (self.store.committed(oid)?, self.store.get(oid)?)
        else {
            return Ok(Vec::new());
        };
        let clashes = |item: &Item| match &item.change {
            Change::Update { before, after } => {
                !clashing_fields(before, after, &committed, &working).is_empty()
            }
            Change::Delete { before } => {
                !fields_the_owner_moved(before, &committed, &working).is_empty()
            }
            Change::Create { .. } => false,
        };
        Ok(self
            .store
            .items()?
            .into_iter()
            .filter(|item| item.is_waiting() && item.target() == oid && clashes(item))
            .map(|item| item.hash)
            .collect())
    }
}

#[cfg(test)]
mod tests;
