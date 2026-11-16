use std::collections::{BTreeMap, BTreeSet};

use dam_domain::{Author, Categories, Context, Date, Object, Oid, Origin};
use jiff::Zoned;

use super::saved::Saved;
use crate::errors::UseCaseError;
use crate::service::Dam;

pub(super) struct Facts<'a> {
    now: Zoned,
    categories: &'a Categories,
    open: BTreeSet<Oid>,
    origins: BTreeMap<Oid, Origin>,
    owner: &'a str,
    saved: Saved,
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

    fn lists(&self, filter: &str, oid: &Oid) -> bool {
        self.saved.lists(filter, oid)
    }

    fn has_event_on(&self, filter: &str, day: Date) -> bool {
        self.saved.has_event_on(filter, day)
    }

    fn author(&self, oid: &Oid) -> &str {
        match self.origins.get(oid).map(|o| &o.author) {
            Some(Author::Token(token)) => &token.name,
            Some(Author::Owner) | None => self.owner,
        }
    }
}
