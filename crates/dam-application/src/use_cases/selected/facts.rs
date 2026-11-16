use std::collections::{BTreeMap, BTreeSet};

use dam_domain::{Author, Categories, Context, Date, Filter, Object, Oid, Origin};
use jiff::Zoned;

use super::pulled::{BeyondPulled, Pulled};
use super::saved::Saved;
use crate::errors::UseCaseError;
use crate::service::Dam;

pub(crate) struct Facts<'a> {
    now: Zoned,
    categories: &'a Categories,
    open: BTreeSet<Oid>,
    origins: BTreeMap<Oid, Origin>,
    owner: &'a str,
    saved: Saved,
    pulled: Pulled,
    unplaced: (usize, usize),
}

impl<'a> Dam<'a> {
    pub(super) fn facts(
        &self,
        categories: &'a Categories,
        seen: &[Object],
        saved: Saved,
    ) -> Result<Facts<'a>, UseCaseError> {
        let open = seen
            .iter()
            .filter_map(Object::as_task)
            .filter(|t| !t.completion.is_done())
            .map(|t| t.base.oid.clone())
            .collect();
        Ok(Facts {
            now: self.clock.now().to_zoned(self.clock.zone()),
            categories,
            open,
            origins: self.origins()?,
            owner: self.config.owner_name(),
            saved,
            pulled: Pulled::of(seen, &self.clock.zone()),
            unplaced: (0, 0),
        })
    }
}

impl Facts<'_> {
    pub(super) fn count_unplaced(&mut self, filter: &Filter, rest: &[Object]) {
        if filter.event_filters().is_empty() {
            return;
        }
        self.pulled.placing(true);
        let placed: Vec<&Object> = rest.iter().filter(|o| filter.matches(o, self)).collect();
        self.pulled.placing(false);
        let tasks = placed.iter().filter(|o| o.as_task().is_some()).count();
        self.unplaced = (tasks, placed.len() - tasks);
    }

    pub(super) fn beyond_pulled(&self, filter: &Filter) -> Option<BeyondPulled> {
        let (tasks, events) = self.unplaced;
        (tasks + events > 0).then(|| BeyondPulled {
            filters: filter
                .event_filters()
                .into_iter()
                .map(String::from)
                .collect(),
            fields: filter.on_fields(),
            held: self.pulled.held,
            tasks,
            events,
        })
    }
}

impl Context for Facts<'_> {
    fn now(&self) -> &Zoned {
        &self.now
    }

    fn categories(&self) -> &Categories {
        self.categories
    }

    fn holds_up(&self, dependency: &Oid) -> bool {
        self.open.contains(dependency)
    }

    fn selects(&self, filter: &str, oid: &Oid) -> bool {
        self.saved.selects(filter, oid)
    }

    fn lists(&self, filter: &str, oid: &Oid) -> bool {
        self.saved.lists(filter, oid)
    }

    fn has_event_on(&self, filter: &str, day: Date) -> bool {
        self.pulled.places_anyway(day) || self.saved.has_event_on(filter, day)
    }

    fn author(&self, oid: &Oid) -> &str {
        match self.origins.get(oid).map(|o| &o.author) {
            Some(Author::Token(token)) => &token.name,
            Some(Author::Owner) | None => self.owner,
        }
    }
}
