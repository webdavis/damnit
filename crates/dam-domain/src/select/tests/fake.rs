use std::collections::{BTreeMap, BTreeSet};

use jiff::Zoned;

use crate::select::{Context, OpenChildren};
use crate::{Categories, Date, Oid};

pub struct Fake {
    pub now: Zoned,
    pub categories: Categories,
    pub open_tasks: BTreeSet<Oid>,
    pub listed: BTreeMap<String, BTreeSet<Oid>>,
    pub selected: BTreeMap<String, BTreeSet<Oid>>,
    pub event_days: BTreeMap<String, BTreeSet<Date>>,
    pub authors: BTreeMap<Oid, String>,
    pub children: OpenChildren,
}

impl Fake {
    pub fn at(now: &str) -> Fake {
        Fake {
            now: now.parse().unwrap(),
            categories: Categories::default(),
            open_tasks: BTreeSet::new(),
            listed: BTreeMap::new(),
            selected: BTreeMap::new(),
            event_days: BTreeMap::new(),
            authors: BTreeMap::new(),
            children: OpenChildren::default(),
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

    fn selects(&self, filter: &str, oid: &Oid) -> bool {
        self.selected.get(filter).is_some_and(|s| s.contains(oid))
    }

    fn lists(&self, filter: &str, oid: &Oid) -> bool {
        self.listed.get(filter).is_some_and(|s| s.contains(oid))
    }

    fn has_event_on(&self, filter: &str, day: Date) -> bool {
        self.event_days
            .get(filter)
            .is_some_and(|s| s.contains(&day))
    }

    fn author(&self, oid: &Oid) -> &str {
        self.authors.get(oid).map_or("owner", String::as_str)
    }

    fn open_children(&self, parent: &Oid) -> usize {
        self.children.of_parent(parent)
    }
}
