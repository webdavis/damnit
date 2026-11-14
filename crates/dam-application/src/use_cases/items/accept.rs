use dam_domain::{Author, Authored, Change, Item, ItemHash, Object, Outcome};

use super::refusal::ItemRefusal;
use crate::errors::UseCaseError;
use crate::service::Dam;
use crate::use_cases::remove::{ItemAndTask, RemovePlan};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Added {
    pub staged: Vec<Authored>,
    pub removals: Vec<AddedRemoval>,
}

impl Added {
    pub(crate) fn extend(&mut self, more: Added) {
        self.staged.extend(more.staged);
        self.removals.extend(more.removals);
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum AddedRemoval {
    Removed { item: ItemHash, plan: RemovePlan },
    AlreadyGone(ItemAndTask),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum OnClash {
    Stop,
    TakeTheirs,
}

impl Dam<'_> {
    pub(crate) fn accept(&self, hash: &ItemHash) -> Result<Added, UseCaseError> {
        self.take(hash, OnClash::Stop)
    }

    pub(crate) fn take(&self, hash: &ItemHash, on_clash: OnClash) -> Result<Added, UseCaseError> {
        let item = self.waiting_item(hash)?;
        let change = match &item.change {
            Change::Create { after } => {
                self.store.put(after)?;
                Some(item.change.clone())
            }
            Change::Update { before, after } => {
                self.apply_change(&item, before, after, on_clash)?
            }
            Change::Delete { before } => return self.take_removal(&item, before, on_clash),
        };
        self.store.settle(hash, Outcome::Added(self.clock.now()))?;
        Ok(Added {
            staged: self.stage_as_sent(&item, change)?.into_iter().collect(),
            removals: Vec::new(),
        })
    }

    fn stage_as_sent(
        &self,
        item: &Item,
        change: Option<Change>,
    ) -> Result<Option<Authored>, UseCaseError> {
        let Some(change) = change else {
            return Ok(None);
        };
        let staged = Authored {
            change,
            author: Author::Token(item.author.clone()),
            details: item.details.clone(),
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

mod follow_ons;
mod merge;
mod removal;

#[cfg(test)]
mod tests;
