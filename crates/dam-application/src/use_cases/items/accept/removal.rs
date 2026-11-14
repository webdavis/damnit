use dam_domain::{Change, Item, Object, Outcome, fields_the_owner_moved};

use super::{Added, AddedRemoval, OnClash};
use crate::errors::UseCaseError;
use crate::service::Dam;
use crate::use_cases::items::refusal::ItemRefusal;
use crate::use_cases::remove::carry_out;

impl Dam<'_> {
    pub(super) fn take_removal(
        &self,
        item: &Item,
        before: &Object,
        on_clash: OnClash,
    ) -> Result<Added, UseCaseError> {
        let committed = self.committed_target(item)?;
        let working = self
            .store
            .get(item.target())?
            .ok_or_else(|| ItemRefusal::TargetGone {
                hash: item.hash.clone(),
                target: item.target().clone(),
            })?;
        let fields = fields_the_owner_moved(before, &committed, &working);
        if on_clash == OnClash::Stop && !fields.is_empty() {
            return Err(ItemRefusal::RemovesAChangedTask {
                hash: item.hash.clone(),
                target: item.target().clone(),
                fields,
            }
            .into());
        }
        let untouched = self.store.all()?;
        let plan = carry_out(self.store, &working)?;
        self.store
            .settle(&item.hash, Outcome::Added(self.clock.now()))?;
        let removal = Change::Delete { before: committed };
        let mut staged: Vec<_> = self
            .stage_as_sent(item, Some(removal))?
            .into_iter()
            .collect();
        staged.extend(self.stage_what_followed(untouched)?);
        Ok(Added {
            staged,
            removals: vec![AddedRemoval::Removed {
                item: item.hash.clone(),
                plan,
            }],
        })
    }
}
