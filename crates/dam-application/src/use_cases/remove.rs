use dam_domain::{Object, Oid};

use crate::errors::UseCaseError;
use crate::ports::ObjectRepository;
use crate::service::Dam;
use crate::use_cases::atomically::atomically;
use crate::use_cases::load::load;
use crate::use_cases::subtree::{children, move_into};

pub struct RemovePlan {
    pub oid: Oid,
    pub moved_up: Vec<Oid>,
    pub dependents: Vec<Oid>,
    pub attached: Vec<Oid>,
}

impl Dam<'_> {
    pub fn plan_remove(&self, oid: &Oid) -> Result<RemovePlan, UseCaseError> {
        plan_for(self.store, &load(self.store, oid)?)
    }

    pub fn remove(&self, oid: &Oid) -> Result<RemovePlan, UseCaseError> {
        atomically(self.store, || {
            let object = load(self.store, oid)?;
            let plan = plan_for(self.store, &object)?;
            for child in &plan.moved_up {
                move_into(self.store, child, &object.base().path)?;
            }
            for dependent in &plan.dependents {
                if let Some(mut d) = self.store.get(dependent)? {
                    d.base_mut().depends.retain(|x| x != oid);
                    self.store.put(&d)?;
                }
            }
            self.store.delete(oid)?;
            Ok(plan)
        })
    }
}

fn plan_for(objects: &dyn ObjectRepository, object: &Object) -> Result<RemovePlan, UseCaseError> {
    let oid = object.oid();
    let moved_up = children(objects, object.base())?
        .iter()
        .map(|o| o.oid().clone())
        .collect();
    let all = objects.all()?;
    let dependents = objects.dependents_of(oid)?;
    let attached = all
        .iter()
        .filter(|o| o.as_task().is_some_and(|t| t.event.as_ref() == Some(oid)))
        .map(|o| o.oid().clone())
        .collect();
    Ok(RemovePlan {
        oid: oid.clone(),
        moved_up,
        dependents,
        attached,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::prelude::*;
    use crate::testing::{FaultyStore, Fixture, MemoryStore, oid};
    use dam_domain::{Event, Object, Path, Task, When};
    use jiff::civil::date;

    fn put(store: &MemoryStore, byte: u8, subject: &str, path: &str) -> Oid {
        let mut t = Task::new(oid(byte), subject);
        t.base.path = Path::parse(path).unwrap();
        store.put(&Object::Task(t)).unwrap();
        oid(byte)
    }

    fn path_of(store: &MemoryStore, id: &Oid) -> String {
        store
            .get(id)
            .unwrap()
            .unwrap()
            .base()
            .path
            .as_str()
            .to_string()
    }

    #[test]
    fn plan_lists_children_dependents_and_attached() {
        let store = MemoryStore::new();
        let plan = put(&store, 1, "plan", "");
        let step = put(&store, 2, "step", "plan");
        put(&store, 3, "detail", "plan/step");
        let d = put(&store, 4, "dependent", "");
        let mut dep = store.get(&d).unwrap().unwrap();
        dep.base_mut().depends.push(plan.clone());
        store.put(&dep).unwrap();
        let fixture = Fixture::default();
        let dam = fixture.dam(&store);
        let found = dam.plan_remove(&plan).unwrap();
        assert_eq!(found.moved_up, vec![step]);
        assert_eq!(found.dependents, vec![d]);
    }

    #[test]
    fn remove_lists_only_the_children_it_moved_up_and_keeps_a_grandchild_under_its_parent() {
        let store = MemoryStore::new();
        let plan = put(&store, 1, "plan", "work");
        let step = put(&store, 2, "step", "work/plan");
        let note = put(&store, 3, "note", "work/plan");
        let detail = put(&store, 4, "detail", "work/plan/step");
        let fixture = Fixture::default();
        let dam = fixture.dam(&store);
        let removed = dam.remove(&plan).unwrap();
        assert_eq!(removed.moved_up, vec![step.clone(), note.clone()]);
        assert_eq!(path_of(&store, &step), "work/");
        assert_eq!(path_of(&store, &note), "work/");
        assert_eq!(path_of(&store, &detail), "work/step/");
    }

    #[test]
    fn remove_moves_children_up_and_drops_dependency_edges() {
        let store = MemoryStore::new();
        let plan = put(&store, 1, "plan", "work");
        let step = put(&store, 2, "step", "work/plan");
        let d = put(&store, 3, "dependent", "");
        let mut dep = store.get(&d).unwrap().unwrap();
        dep.base_mut().depends.push(plan.clone());
        store.put(&dep).unwrap();
        let fixture = Fixture::default();
        let dam = fixture.dam(&store);
        dam.remove(&plan).unwrap();
        assert!(store.get(&plan).unwrap().is_none());
        assert_eq!(path_of(&store, &step), "work/");
        assert!(store.get(&d).unwrap().unwrap().base().depends.is_empty());
    }

    #[test]
    fn removing_a_task_leaves_objects_in_other_folders_alone() {
        let store = MemoryStore::new();
        let offsite = put(&store, 1, "plan the offsite", "work");
        let note = put(&store, 2, "unrelated project note", "work/clients");
        let another = put(&store, 3, "another", "work/clients/acme");
        let fixture = Fixture::default();
        let dam = fixture.dam(&store);
        let plan = dam.remove(&offsite).unwrap();
        assert!(plan.moved_up.is_empty());
        assert_eq!(path_of(&store, &note), "work/clients/");
        assert_eq!(path_of(&store, &another), "work/clients/acme/");
    }

    #[test]
    fn an_attached_task_keeps_the_removed_events_oid() {
        let store = MemoryStore::new();
        let day = When::Day(date(2026, 9, 25));
        store
            .put(&Object::Event(Event::new(
                oid(5),
                "meeting",
                day.clone(),
                day,
            )))
            .unwrap();
        let mut prep = Task::new(oid(6), "prep");
        prep.event = Some(oid(5));
        store.put(&Object::Task(prep)).unwrap();
        let fixture = Fixture::default();
        let dam = fixture.dam(&store);
        let plan = dam.remove(&oid(5)).unwrap();
        assert_eq!(plan.attached, vec![oid(6)]);
        let kept = store.get(&oid(6)).unwrap().unwrap();
        assert_eq!(kept.as_task().unwrap().event, Some(oid(5)));
    }

    #[test]
    fn rm_that_fails_between_two_children_moves_neither() {
        let store = FaultyStore::new(MemoryStore::new());
        let plan = put(&store.inner, 1, "p", "top");
        put(&store.inner, 2, "a", "top/p");
        put(&store.inner, 3, "b", "top/p");
        let fixture = Fixture::default();
        let dam = fixture.dam(&store);
        store.assert_unchanged_when_any_write_fails(|| dam.remove(&plan).map(drop));
    }
}
