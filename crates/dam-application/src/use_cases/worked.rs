use std::collections::BTreeSet;

use dam_domain::{
    Categories, Context, Date, Object, Oid, OpenChildren, Path, WorkedOut, worked_out,
};
use jiff::Zoned;

use crate::errors::UseCaseError;
use crate::ports::Handed;
use crate::service::Dam;

struct Worked<'a> {
    now: &'a Zoned,
    categories: &'a Categories,
    open: &'a BTreeSet<Oid>,
    children: &'a OpenChildren,
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

    fn open_children(&self, parent: &Oid) -> usize {
        self.children.of_parent(parent)
    }
}

impl Dam<'_> {
    pub fn worked_out_from_object(&self, object: &Object) -> WorkedOut {
        WorkedOut {
            blocked: None,
            children_open: None,
            ..self.worked_out_among(object, &BTreeSet::new(), &OpenChildren::default())
        }
    }

    pub(crate) fn open_children(&self) -> Result<OpenChildren, UseCaseError> {
        let seen = self.seen_within(&Path::root())?;
        Ok(OpenChildren::of(seen.iter().filter_map(Object::as_task)))
    }

    pub(crate) fn handed(
        &self,
        object: Object,
        children: &OpenChildren,
    ) -> Result<Handed, UseCaseError> {
        let mut open = BTreeSet::new();
        for dependency in &object.base().depends {
            if let Some(Object::Task(task)) = self.seen_object(dependency)?
                && !task.completion.is_done()
            {
                open.insert(task.base.oid);
            }
        }
        let worked_out = self.worked_out_among(&object, &open, children);
        Ok(Handed { object, worked_out })
    }

    pub(crate) fn worked_out_over_seen(
        &self,
        objects: &[Object],
    ) -> Result<Vec<WorkedOut>, UseCaseError> {
        let seen = self.seen_within(&Path::root())?;
        let open = open_tasks(&seen);
        let children = OpenChildren::of(seen.iter().filter_map(Object::as_task));
        Ok(objects
            .iter()
            .map(|o| self.worked_out_among(o, &open, &children))
            .collect())
    }

    fn worked_out_among(
        &self,
        object: &Object,
        open: &BTreeSet<Oid>,
        children: &OpenChildren,
    ) -> WorkedOut {
        let now = self.clock.now().to_zoned(self.clock.zone());
        let worked = Worked {
            now: &now,
            categories: self.config.categories_at(&object.base().path),
            open,
            children,
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
