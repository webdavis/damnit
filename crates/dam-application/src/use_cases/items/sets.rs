use dam_domain::{Item, ItemHash};

use super::accept::{Added, OnClash};
use super::refusal::ItemRefusal;
use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;
use crate::use_cases::atomically::atomically;

impl Dam<'_> {
    pub(crate) fn alone(&self, hash: &ItemHash) -> Result<(), UseCaseError> {
        match self.store.item(hash)?.and_then(|item| item.set) {
            Some(set) => Err(ItemRefusal::InASet(set).into()),
            None => Ok(()),
        }
    }

    pub(crate) fn waiting_set(&self, set: &ItemHash) -> Result<Vec<Item>, UseCaseError> {
        let members: Vec<Item> = self
            .store
            .items()?
            .into_iter()
            .filter(|item| item.is_waiting() && item.set.as_ref() == Some(set))
            .collect();
        if members.is_empty() || !members.iter().all(|item| self.sees_item(item)) {
            return Err(Refusal::NoSuchObject(set.short().to_string()).into());
        }
        Ok(members)
    }

    pub fn waiting_sets(&self) -> Result<Vec<ItemHash>, UseCaseError> {
        let mut sets: Vec<ItemHash> = Vec::new();
        for item in self.store.items()? {
            if item.is_waiting()
                && let Some(set) = item.set
                && !sets.contains(&set)
                && self.waiting_set(&set).is_ok()
            {
                sets.push(set);
            }
        }
        Ok(sets)
    }

    pub fn set_items(&self, set: &ItemHash) -> Result<Vec<Item>, UseCaseError> {
        self.waiting_set(set)?;
        Ok(self
            .waiting_items()?
            .into_iter()
            .filter(|item| item.set.as_ref() == Some(set))
            .collect())
    }

    pub(crate) fn take_set(
        &self,
        set: &ItemHash,
        on_clash: OnClash,
    ) -> Result<Added, UseCaseError> {
        atomically(self.store, || {
            let mut added = Added::default();
            for member in self.waiting_set(set)? {
                added.extend(self.take(&member.hash, on_clash)?);
            }
            Ok(added)
        })
    }

    pub fn dismiss_set(&self, set: &ItemHash) -> Result<Vec<Item>, UseCaseError> {
        atomically(self.store, || {
            self.waiting_set(set)?
                .iter()
                .map(|member| self.dismiss_member(&member.hash))
                .collect()
        })
    }
}

#[cfg(test)]
mod tests;
