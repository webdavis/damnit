use dam_domain::Oid;

use crate::errors::{Refusal, UseCaseError};
use crate::ports::ObjectRepository;
use crate::use_cases::atomically::atomically;
use crate::use_cases::subtree::{children, move_into};

pub struct RemovePlan {
    pub oid: Oid,
    pub descendants: Vec<Oid>,
    pub dependents: Vec<Oid>,
    pub attached: Vec<Oid>,
}

pub fn plan_remove(objects: &dyn ObjectRepository, oid: &Oid) -> Result<RemovePlan, UseCaseError> {
    let object = objects
        .get(oid)?
        .ok_or_else(|| Refusal::NoSuchObject(oid.short().to_string()))?;
    let owned = object.base().owned_folder();
    let all = objects.all()?;
    let descendants = all
        .iter()
        .filter(|o| owned.as_ref().is_some_and(|f| o.base().path.is_within(f)))
        .map(|o| o.oid().clone())
        .collect();
    let dependents = objects.dependents_of(oid)?;
    let attached = all
        .iter()
        .filter(|o| o.as_task().is_some_and(|t| t.event.as_ref() == Some(oid)))
        .map(|o| o.oid().clone())
        .collect();
    Ok(RemovePlan {
        oid: oid.clone(),
        descendants,
        dependents,
        attached,
    })
}

pub fn remove(objects: &dyn ObjectRepository, oid: &Oid) -> Result<RemovePlan, UseCaseError> {
    atomically(objects, || {
        let plan = plan_remove(objects, oid)?;
        let object = objects
            .get(oid)?
            .ok_or_else(|| Refusal::NoSuchObject(oid.short().to_string()))?;
        for child in children(objects, object.base())? {
            move_into(objects, child.oid(), &object.base().path)?;
        }
        for dependent in &plan.dependents {
            if let Some(mut d) = objects.get(dependent)? {
                d.base_mut().depends.retain(|x| x != oid);
                objects.put(&d)?;
            }
        }
        objects.delete(oid)?;
        Ok(plan)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::prelude::*;
    use crate::testing::{FaultyStore, MemoryStore, oid};
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
    fn plan_lists_descendants_dependents_and_attached() {
        let store = MemoryStore::new();
        let plan = put(&store, 1, "plan", "");
        let step = put(&store, 2, "step", "plan");
        let detail = put(&store, 3, "detail", "plan/step");
        let d = put(&store, 4, "dependent", "");
        let mut dep = store.get(&d).unwrap().unwrap();
        dep.base_mut().depends.push(plan.clone());
        store.put(&dep).unwrap();
        let found = plan_remove(&store, &plan).unwrap();
        assert_eq!(found.descendants, vec![step, detail]);
        assert_eq!(found.dependents, vec![d]);
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
        remove(&store, &plan).unwrap();
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
        let plan = remove(&store, &offsite).unwrap();
        assert!(plan.descendants.is_empty());
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
        let plan = remove(&store, &oid(5)).unwrap();
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
        store.assert_unchanged_when_any_write_fails(|| remove(&store, &plan).map(drop));
    }
}
