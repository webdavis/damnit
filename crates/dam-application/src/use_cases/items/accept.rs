use dam_domain::{Author, Authored, Change, Item, ItemHash, Object, Outcome};

use super::refusal::ItemRefusal;
use crate::errors::UseCaseError;
use crate::service::Dam;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum OnClash {
    Stop,
    TakeTheirs,
}

impl Dam<'_> {
    pub(crate) fn accept(&self, hash: &ItemHash) -> Result<Option<Authored>, UseCaseError> {
        self.take(hash, OnClash::Stop)
    }

    pub(crate) fn take(
        &self,
        hash: &ItemHash,
        on_clash: OnClash,
    ) -> Result<Option<Authored>, UseCaseError> {
        let item = self.waiting_item(hash)?;
        let change = match &item.change {
            Change::Create { after } => {
                self.store.put(after)?;
                Some(item.change.clone())
            }
            Change::Update { before, after } => {
                self.apply_change(&item, before, after, on_clash)?
            }
            Change::Delete { before } => self.apply_removal(&item, before, on_clash)?,
        };
        self.store.settle(hash, Outcome::Added(self.clock.now()))?;
        let Some(change) = change else {
            return Ok(None);
        };
        let staged = Authored {
            change,
            author: Author::Token(item.author),
            details: item.details,
        };
        self.store.put_staged(staged.clone())?;
        Ok(Some(staged))
    }

    pub(crate) fn waiting_item(&self, hash: &ItemHash) -> Result<Item, UseCaseError> {
        let item = self
            .store
            .item(hash)?
            .ok_or_else(|| crate::errors::Refusal::NoSuchObject(hash.short().to_string()))?;
        if !item.is_waiting() {
            return Err(ItemRefusal::NotWaiting {
                hash: item.hash,
                outcome: item.outcome.as_str(),
            }
            .into());
        }
        Ok(item)
    }

    pub(crate) fn committed_target(&self, item: &Item) -> Result<Object, UseCaseError> {
        let target = item.target();
        if self
            .store
            .staged()?
            .iter()
            .any(|s| s.change.oid() == target)
        {
            return Err(ItemRefusal::TargetStaged {
                hash: item.hash.clone(),
                target: target.clone(),
            }
            .into());
        }
        self.store.committed(target)?.ok_or_else(|| {
            ItemRefusal::TargetNotCommitted {
                hash: item.hash.clone(),
                target: target.clone(),
            }
            .into()
        })
    }
}

mod merge;

#[cfg(test)]
mod tests;
