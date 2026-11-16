use std::collections::{BTreeMap, BTreeSet};

use dam_domain::{Date, Filter, Oid, Path};

use super::event_days::event_days;
use crate::config::FilterConfig;
use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;

#[derive(Default)]
pub(super) struct Saved {
    listed: BTreeMap<String, BTreeSet<Oid>>,
    event_days: BTreeMap<String, BTreeSet<Date>>,
}

impl Saved {
    pub(super) fn lists(&self, filter: &str, oid: &Oid) -> bool {
        self.listed.get(filter).is_some_and(|s| s.contains(oid))
    }

    pub(super) fn has_event_on(&self, filter: &str, day: Date) -> bool {
        self.event_days
            .get(filter)
            .is_some_and(|s| s.contains(&day))
    }
}

impl Dam<'_> {
    pub(super) fn saved(
        &self,
        filter: &Filter,
        place: &Path,
        local: &[FilterConfig],
    ) -> Result<Saved, UseCaseError> {
        let mut saved = Saved::default();
        let zone = self.clock.zone();
        for name in filter.saved_filters() {
            let listed = match local.iter().find(|f| f.name == name) {
                Some(here) => self.list_matching(here.rule.query(), None, place)?,
                None => {
                    let config = self
                        .config
                        .filter(name)
                        .ok_or_else(|| Refusal::NoSuchFilter(name.into()))?;
                    self.list_matching(config.rule.query(), Some(config), &Path::root())?
                }
            };
            let oids = listed.iter().map(|o| o.oid().clone()).collect();
            let days = listed
                .iter()
                .filter_map(|o| o.as_event())
                .flat_map(|e| event_days(e, &zone))
                .collect();
            saved.listed.insert(name.into(), oids);
            saved.event_days.insert(name.into(), days);
        }
        Ok(saved)
    }
}
