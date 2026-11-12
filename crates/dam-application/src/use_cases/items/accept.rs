use dam_domain::{
    Author, Authored, Change, Item, ItemHash, Object, Outcome, changed_fields, clashing_fields,
    fields_the_owner_moved, merge_fields,
};

use super::refusal::ItemRefusal;
use crate::errors::UseCaseError;
use crate::service::Dam;
use crate::use_cases::subtree::{loses_a_held_folder, replace_carrying_folder};

impl Dam<'_> {
    pub(crate) fn accept(&self, hash: &ItemHash) -> Result<Option<Authored>, UseCaseError> {
        let item = self.waiting_item(hash)?;
        let change = match &item.change {
            Change::Create { after } => {
                self.store.put(after)?;
                Some(item.change.clone())
            }
            Change::Update { before, after } => self.apply_change(&item, before, after)?,
            Change::Delete { before } => self.apply_removal(&item, before)?,
        };
        self.store.settle(hash, Outcome::Added(self.clock.now()))?;
        let Some(change) = change else {
            return Ok(None);
        };
        let staged = Authored {
            change,
            author: Author::Token(item.author),
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

    fn apply_change(
        &self,
        item: &Item,
        before: &Object,
        after: &Object,
    ) -> Result<Option<Change>, UseCaseError> {
        let committed = self.committed_target(item)?;
        let working = self
            .store
            .get(item.target())?
            .ok_or_else(|| ItemRefusal::TargetGone {
                hash: item.hash.clone(),
                target: item.target().clone(),
            })?;
        let clash = clashing_fields(before, after, &committed, &working);
        if !clash.is_empty() {
            return Err(ItemRefusal::Clash {
                hash: item.hash.clone(),
                target: item.target().clone(),
                fields: clash,
            }
            .into());
        }
        self.take_theirs(item, &committed, &working, after)
    }

    pub(crate) fn take_theirs(
        &self,
        item: &Item,
        committed: &Object,
        working: &Object,
        after: &Object,
    ) -> Result<Option<Change>, UseCaseError> {
        let fields = changed_fields(item.change.before().unwrap_or(after), after);
        let merged = merge_fields(working, after, &fields);
        if loses_a_held_folder(self.store, working, &merged)? {
            return Err(ItemRefusal::SubjectNamesNoFolder {
                hash: item.hash.clone(),
                target: item.target().clone(),
            }
            .into());
        }
        replace_carrying_folder(self.store, working, &merged)?;
        let staged = merge_fields(committed, after, &fields);
        Ok(Change::between(Some(committed.clone()), Some(staged)))
    }

    fn apply_removal(&self, item: &Item, before: &Object) -> Result<Option<Change>, UseCaseError> {
        let committed = self.committed_target(item)?;
        if let Some(working) = self.store.get(item.target())? {
            let fields = fields_the_owner_moved(before, &committed, &working);
            if !fields.is_empty() {
                return Err(ItemRefusal::Clash {
                    hash: item.hash.clone(),
                    target: item.target().clone(),
                    fields,
                }
                .into());
            }
        }
        self.store.delete(item.target())?;
        Ok(Some(Change::Delete { before: committed }))
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

#[cfg(test)]
mod tests;
