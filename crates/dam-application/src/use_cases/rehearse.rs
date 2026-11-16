use std::collections::{BTreeMap, BTreeSet};

use dam_domain::{Change, Object, Oid};

use crate::config::Config;
use crate::errors::UseCaseError;
use crate::service::Dam;
use crate::use_cases::each::rehearsal::rehearsed;

pub struct Rehearsed<T> {
    pub ran: T,
    pub changes: Vec<Change>,
}

impl Dam<'_> {
    pub fn rehearse<T>(
        &self,
        work: impl FnOnce(&Dam<'_>) -> Result<T, UseCaseError>,
    ) -> Result<Rehearsed<T>, UseCaseError> {
        let config = Config {
            hooks: Vec::new(),
            ..self.config.clone()
        };
        let without_hooks = Dam {
            config: &config,
            ..*self
        };
        let mut before = self.seen_objects()?;
        rehearsed(self.store, || {
            let ran = work(&without_hooks)?;
            let mut after = self.seen_objects()?;
            let oids: BTreeSet<Oid> = before.keys().chain(after.keys()).cloned().collect();
            let mut changes = Vec::new();
            for oid in oids {
                if let Some(change) = Change::between(before.remove(&oid), after.remove(&oid)) {
                    changes.push(self.as_seen_change(change)?);
                }
            }
            Ok(Rehearsed { ran, changes })
        })
    }

    fn seen_objects(&self) -> Result<BTreeMap<Oid, Object>, UseCaseError> {
        Ok(self
            .store
            .all()?
            .into_iter()
            .filter(|o| self.access.sees(o))
            .map(|o| (o.oid().clone(), o))
            .collect())
    }
}

#[cfg(test)]
mod tests;
