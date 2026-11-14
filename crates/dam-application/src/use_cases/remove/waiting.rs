use std::collections::BTreeMap;

use dam_domain::{Change, Item, ItemHash};

use super::{RemovePlan, plan_for};
use crate::errors::UseCaseError;
use crate::service::Dam;

impl Dam<'_> {
    pub(crate) fn plans_of_waiting_removals(
        &self,
        waiting: &[Item],
    ) -> Result<BTreeMap<ItemHash, RemovePlan>, UseCaseError> {
        let mut plans = BTreeMap::new();
        if self.access.token().is_some() {
            return Ok(plans);
        }
        for item in waiting {
            if let Change::Delete { .. } = item.change
                && let Some(object) = self.store.get(item.target())?
            {
                plans.insert(item.hash.clone(), plan_for(self.store, &object)?);
            }
        }
        Ok(plans)
    }
}
