use dam_domain::{Item, Object, Oid};

use super::refusal::ItemRefusal;
use crate::errors::UseCaseError;
use crate::service::Dam;

impl Dam<'_> {
    pub(super) fn refuse_what_the_owners_edit_or_done_would(
        &self,
        item: &Item,
        working: &Object,
        merged: &Object,
    ) -> Result<(), UseCaseError> {
        let added: Vec<Oid> = merged
            .base()
            .depends
            .iter()
            .filter(|d| !working.base().depends.contains(d))
            .cloned()
            .collect();
        if let Some(path) = self.cycle_through(item.target(), &added)? {
            return Err(ItemRefusal::Cycle {
                hash: item.hash.clone(),
                target: item.target().clone(),
                path,
            }
            .into());
        }
        if completes(working, merged)
            && let Some(task) = merged.as_task()
        {
            let blockers = self.blockers_of(task)?;
            if !blockers.is_empty() {
                return Err(ItemRefusal::Blocked {
                    hash: item.hash.clone(),
                    target: item.target().clone(),
                    blockers,
                }
                .into());
            }
        }
        Ok(())
    }
}

fn completes(before: &Object, after: &Object) -> bool {
    let done = |o: &Object| o.as_task().is_some_and(|t| t.completion.is_done());
    !done(before) && done(after)
}
