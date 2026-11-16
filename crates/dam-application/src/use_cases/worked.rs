use std::collections::BTreeSet;

use dam_domain::{Categories, Context, Date, Object, Oid, Path, WorkedOut, worked_out};
use jiff::Zoned;

use crate::errors::UseCaseError;
use crate::service::Dam;

struct Worked<'a> {
    now: &'a Zoned,
    categories: &'a Categories,
    open: &'a BTreeSet<Oid>,
    owner: &'a str,
}

impl Context for Worked<'_> {
    fn now(&self) -> &Zoned {
        self.now
    }

    fn categories(&self) -> &Categories {
        self.categories
    }

    fn holds_up(&self, dependency: &Oid) -> bool {
        self.open.contains(dependency)
    }

    fn selects(&self, _: &str, _: &Oid) -> bool {
        false
    }

    fn lists(&self, _: &str, _: &Oid) -> bool {
        false
    }

    fn has_event_on(&self, _: &str, _: Date) -> bool {
        false
    }

    fn author(&self, _: &Oid) -> &str {
        self.owner
    }
}

impl Dam<'_> {
    pub fn worked_out_from_object(&self, object: &Object) -> WorkedOut {
        WorkedOut {
            blocked: None,
            ..self.worked_out_among(object, &BTreeSet::new())
        }
    }

    pub(crate) fn worked_out_over_seen(
        &self,
        objects: &[Object],
    ) -> Result<Vec<WorkedOut>, UseCaseError> {
        let open = open_tasks(&self.seen_within(&Path::root())?);
        Ok(objects
            .iter()
            .map(|o| self.worked_out_among(o, &open))
            .collect())
    }

    fn worked_out_among(&self, object: &Object, open: &BTreeSet<Oid>) -> WorkedOut {
        let now = self.clock.now().to_zoned(self.clock.zone());
        let worked = Worked {
            now: &now,
            categories: self.config.categories_at(&object.base().path),
            open,
            owner: self.config.owner_name(),
        };
        worked_out(object, &worked)
    }
}

fn open_tasks(seen: &[Object]) -> BTreeSet<Oid> {
    seen.iter()
        .filter_map(Object::as_task)
        .filter(|t| !t.completion.is_done())
        .map(|t| t.base.oid.clone())
        .collect()
}
