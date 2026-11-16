use std::collections::BTreeSet;

use dam_domain::{LabelViolation, Object, Oid};

use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;
use crate::use_cases::each::{Selector, SetChanges, SetRun, sent};
use crate::use_cases::edit::EditFields;
use crate::use_cases::load::no_such_object;

pub use map::{LabelMap, MapProblem};
pub use places::{ConfigPlace, config_places};

impl Dam<'_> {
    pub fn migrate_labels(&self, map: &LabelMap, run: SetRun) -> Result<SetChanges, UseCaseError> {
        if self.access.token().is_some() {
            return Err(Refusal::NotOpenToTokens("category migrate").into());
        }
        let carrying: Vec<Object> = self
            .store
            .all()?
            .into_iter()
            .filter(|object| map.touches(&object.base().labels))
            .collect();
        self.refuse_a_clash(&carrying, map)?;
        let oids: Vec<Oid> = carrying.iter().map(|o| o.oid().clone()).collect();
        self.each(&Selector::Oids(oids), run, false, |oid| {
            let (remove_labels, add_labels) = match self.store.get(oid)? {
                Some(object) => map.swaps(&object.base().labels),
                None => return Err(no_such_object(oid).into()),
            };
            let fields = EditFields {
                remove_labels,
                add_labels,
                ..EditFields::default()
            };
            Ok(sent(self.edit_in_place(oid, fields)?))
        })
    }

    fn refuse_a_clash(&self, carrying: &[Object], map: &LabelMap) -> Result<(), UseCaseError> {
        let mut clash: Option<(String, Vec<Oid>)> = None;
        for object in carrying {
            let after: BTreeSet<String> = map.applied(&object.base().labels);
            if let Err(LabelViolation::Exclusive { category, .. }) =
                self.config.categories.check(&after)
            {
                clash
                    .get_or_insert_with(|| (category, Vec::new()))
                    .1
                    .push(object.oid().clone());
            }
        }
        match clash {
            Some((category, oids)) => Err(Refusal::ExclusiveAfterMigrate { category, oids }.into()),
            None => Ok(()),
        }
    }
}

mod map;
mod places;

#[cfg(test)]
mod tests;
