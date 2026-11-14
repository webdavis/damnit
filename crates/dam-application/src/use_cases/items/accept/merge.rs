use dam_domain::{
    Change, Item, Object, changed_fields, clashing_fields, fields_the_owner_moved, merge_fields,
};

use super::OnClash;
use crate::errors::UseCaseError;
use crate::service::Dam;
use crate::use_cases::items::refusal::ItemRefusal;
use crate::use_cases::subtree::{loses_a_held_folder, replace_carrying_folder};

impl Dam<'_> {
    pub(super) fn apply_change(
        &self,
        item: &Item,
        before: &Object,
        after: &Object,
        on_clash: OnClash,
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
        if !clash.is_empty() && on_clash == OnClash::Stop {
            return Err(ItemRefusal::Clash {
                hash: item.hash.clone(),
                target: item.target().clone(),
                fields: clash,
            }
            .into());
        }
        let fields = changed_fields(before, after);
        let merged = merge_fields(&working, after, &fields);
        self.refuse_what_the_owners_edit_or_done_would(item, &working, &merged)?;
        if loses_a_held_folder(self.store, &working, &merged)? {
            return Err(ItemRefusal::SubjectNamesNoFolder {
                hash: item.hash.clone(),
                target: item.target().clone(),
            }
            .into());
        }
        replace_carrying_folder(self.store, &working, &merged)?;
        let staged = merge_fields(&committed, after, &fields);
        Ok(Change::between(Some(committed), Some(staged)))
    }

    pub(super) fn apply_removal(
        &self,
        item: &Item,
        before: &Object,
        on_clash: OnClash,
    ) -> Result<Option<Change>, UseCaseError> {
        let committed = self.committed_target(item)?;
        if on_clash == OnClash::Stop
            && let Some(working) = self.store.get(item.target())?
        {
            let fields = fields_the_owner_moved(before, &committed, &working);
            if !fields.is_empty() {
                return Err(ItemRefusal::RemovesAChangedTask {
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
}
