use dam_domain::Oid;

use crate::errors::{Refusal, UseCaseError};
use crate::ports::ObjectStore;
use crate::use_cases::subtree::{last_segment, move_subtree};

pub struct RemovePlan {
    pub oid: Oid,
    pub descendants: Vec<Oid>,
    pub dependents: Vec<Oid>,
    pub attached: Vec<Oid>,
}

pub fn plan_remove(store: &dyn ObjectStore, oid: &Oid) -> Result<RemovePlan, UseCaseError> {
    let object = store
        .get(oid)?
        .ok_or_else(|| Refusal::NoSuchObject(oid.short().to_string()))?;
    let path = object.base().path.clone();
    let all = store.all()?;
    let descendants = all
        .iter()
        .filter(|o| o.oid() != oid && o.base().path.is_within(&path))
        .map(|o| o.oid().clone())
        .collect();
    let dependents = store.dependents_of(oid)?;
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

/// Removes one object. Children move up a level, dependents lose the edge,
/// attached tasks keep the oid so `status` can report it. Nothing cascades.
pub fn remove(store: &dyn ObjectStore, oid: &Oid) -> Result<RemovePlan, UseCaseError> {
    let plan = plan_remove(store, oid)?;
    let object = store
        .get(oid)?
        .ok_or_else(|| Refusal::NoSuchObject(oid.short().to_string()))?;
    let old = object.base().path.clone();
    let up = old.parent().unwrap_or_default();
    for child in store.children_of(&old)? {
        let new = up
            .join(&last_segment(&child.base().path))
            .map_err(|e| UseCaseError::Parse(e.to_string()))?;
        move_subtree(store, child.oid(), &new)?;
    }
    for dependent in &plan.dependents {
        if let Some(mut d) = store.get(dependent)? {
            d.base_mut().depends.retain(|x| x != oid);
            store.put(&d)?;
        }
    }
    store.delete(oid)?;
    Ok(plan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{MemoryStore, oid};
    use dam_domain::{Object, Path, Task};

    fn put(store: &MemoryStore, byte: u8, path: &str) -> Oid {
        let mut t = Task::new(oid(byte), format!("t{byte}"));
        t.base.path = Path::parse(path).unwrap();
        store.put(&Object::Task(t)).unwrap();
        oid(byte)
    }

    #[test]
    fn plan_lists_descendants_dependents_and_attached() {
        let store = MemoryStore::new();
        let p = put(&store, 1, "p");
        let c = put(&store, 2, "p/c");
        let d = put(&store, 3, "");
        let mut dep = store.get(&d).unwrap().unwrap();
        dep.base_mut().depends.push(p.clone());
        store.put(&dep).unwrap();
        let plan = plan_remove(&store, &p).unwrap();
        assert_eq!(plan.descendants, vec![c]);
        assert_eq!(plan.dependents, vec![d]);
    }

    #[test]
    fn remove_moves_children_up_and_drops_dependency_edges() {
        let store = MemoryStore::new();
        let p = put(&store, 1, "top/p");
        let c = put(&store, 2, "top/p/c");
        let d = put(&store, 3, "");
        let mut dep = store.get(&d).unwrap().unwrap();
        dep.base_mut().depends.push(p.clone());
        store.put(&dep).unwrap();
        remove(&store, &p).unwrap();
        assert!(store.get(&p).unwrap().is_none());
        assert_eq!(
            store.get(&c).unwrap().unwrap().base().path.as_str(),
            "top/c/"
        );
        assert!(store.get(&d).unwrap().unwrap().base().depends.is_empty());
    }
}
