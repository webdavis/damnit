use std::collections::{BTreeMap, BTreeSet};

use dam_domain::{Command, Date, Filter, FilterTable, Object, Oid, Path, Scope, TableValue};

use super::event_days::event_days;
use super::{Defaults, SelectError};
use crate::config::FilterConfig;
use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;

const EVERYTHING: Scope = Scope {
    tasks: true,
    events: true,
    done: None,
};
const EVENTS: Scope = Scope {
    tasks: false,
    events: true,
    done: None,
};

#[derive(Default)]
pub(super) struct Saved {
    selected: BTreeMap<String, BTreeSet<Oid>>,
    listed: BTreeMap<String, BTreeSet<Oid>>,
    event_days: BTreeMap<String, BTreeSet<Date>>,
}

impl Saved {
    pub(super) fn selects(&self, filter: &str, oid: &Oid) -> bool {
        self.selected.get(filter).is_some_and(|s| s.contains(oid))
    }

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
    pub(crate) fn listed_by_name(
        &self,
        name: &str,
        place: &Path,
        local: &[FilterConfig],
    ) -> Result<Vec<Object>, UseCaseError> {
        let named = FilterTable {
            fields: [("filter".into(), TableValue::One(name.into()))].into(),
            any: Vec::new(),
        };
        let filter = Filter::read(&named).map_err(SelectError::Value)?;
        Ok(self.selected(&filter, Command::List, place, local)?.0)
    }

    pub(crate) fn listed_by_table(
        &self,
        name: &str,
        defaults: Defaults,
        (place, local): (&Path, &[FilterConfig]),
        chain: &[String],
    ) -> Result<(FilterConfig, Vec<Object>), UseCaseError> {
        let mut chain = chain.to_vec();
        chain.push(name.to_string());
        if chain[..chain.len() - 1].iter().any(|n| n == name) {
            return Err(Refusal::FilterReachesItself(chain).into());
        }
        let (config, home, reachable) = self.saved_config(name, place, local)?;
        let filter = Filter::read(&config.table.under(&home)).map_err(SelectError::Value)?;
        let (listed, _) = self.chosen(&filter, defaults, &home, reachable, &chain)?;
        Ok((config.clone(), listed))
    }

    pub(super) fn saved_config<'l>(
        &'l self,
        name: &str,
        place: &Path,
        local: &'l [FilterConfig],
    ) -> Result<(&'l FilterConfig, Path, &'l [FilterConfig]), UseCaseError> {
        if let Some(here) = local.iter().find(|f| f.name == name) {
            return Ok((here, place.clone(), local));
        }
        let config = self
            .config
            .filter(name)
            .ok_or_else(|| Refusal::NoSuchFilter(name.into()))?;
        Ok((config, Path::root(), &[]))
    }

    fn listed_by_saved(
        &self,
        name: &str,
        defaults: Defaults,
        at: (&Path, &[FilterConfig]),
        chain: &[String],
    ) -> Result<Vec<Object>, UseCaseError> {
        let (config, listed) = self.listed_by_table(name, defaults, at, chain)?;
        self.kept(&config, listed)
    }

    pub(super) fn saved(
        &self,
        filter: &Filter,
        at: (&Path, &[FilterConfig]),
        chain: &[String],
    ) -> Result<Saved, UseCaseError> {
        let mut saved = Saved::default();
        let oids = |listed: Vec<Object>| listed.iter().map(|o| o.oid().clone()).collect();
        for name in filter.selected_filters() {
            let listed = self.listed_by_saved(name, Defaults::Fixed(EVERYTHING), at, chain)?;
            saved.selected.insert(name.into(), oids(listed));
        }
        for name in filter.listed_filters() {
            let listed = self.listed_by_saved(name, Defaults::Of(Command::List), at, chain)?;
            saved.listed.insert(name.into(), oids(listed));
        }
        let zone = self.clock.zone();
        for name in filter.event_filters() {
            let listed = self.listed_by_saved(name, Defaults::Fixed(EVENTS), at, chain)?;
            let days = listed
                .iter()
                .filter_map(|o| o.as_event())
                .flat_map(|e| event_days(e, &zone))
                .collect();
            saved.event_days.insert(name.into(), days);
        }
        Ok(saved)
    }
}
