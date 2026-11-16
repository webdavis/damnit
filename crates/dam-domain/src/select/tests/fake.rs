use std::collections::{BTreeMap, BTreeSet};

use jiff::Zoned;

use crate::select::Context;
use crate::{Categories, Date, Oid};

pub struct Fake {
    pub now: Zoned,
    pub categories: Categories,
    pub open_tasks: BTreeSet<Oid>,
    pub listed: BTreeMap<String, BTreeSet<Oid>>,
    pub event_days: BTreeMap<String, BTreeSet<Date>>,
}

impl Fake {
    pub fn at(now: &str) -> Fake {
        Fake {
            now: now.parse().unwrap(),
            categories: Categories::default(),
            open_tasks: BTreeSet::new(),
            listed: BTreeMap::new(),
            event_days: BTreeMap::new(),
        }
    }
}

impl Context for Fake {
    fn now(&self) -> &Zoned {
        &self.now
    }

    fn categories(&self) -> &Categories {
        &self.categories
    }

    fn holds_up(&self, dependency: &Oid) -> bool {
        self.open_tasks.contains(dependency)
    }

    fn lists(&self, filter: &str, oid: &Oid) -> bool {
        self.listed.get(filter).is_some_and(|s| s.contains(oid))
    }

    fn has_event_on(&self, filter: &str, day: Date) -> bool {
        self.event_days
            .get(filter)
            .is_some_and(|s| s.contains(&day))
    }
}
