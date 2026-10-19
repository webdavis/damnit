use dam_domain::{Base, Object, Oid, Path};

use crate::errors::{Refusal, UseCaseError};
use crate::ports::ObjectRepository;

pub(crate) fn children(
    objects: &dyn ObjectRepository,
    owner: &Base,
) -> Result<Vec<Object>, UseCaseError> {
    let Some(folder) = owner.owned_folder() else {
        return Ok(Vec::new());
    };
    Ok(objects
        .all()?
        .into_iter()
        .filter(|o| o.base().path == folder)
        .collect())
}

pub(crate) fn move_into(
    objects: &dyn ObjectRepository,
    oid: &Oid,
    folder: &Path,
) -> Result<(), UseCaseError> {
    let mut object = objects
        .get(oid)?
        .ok_or_else(|| Refusal::NoSuchObject(oid.short().to_string()))?;
    let owned_before = object.base().owned_folder();
    object.base_mut().path = folder.clone();
    if let (Some(from), Some(to)) = (owned_before, object.base().owned_folder()) {
        move_folder(objects, &from, &to)?;
    }
    objects.put(&object)?;
    Ok(())
}

fn move_folder(objects: &dyn ObjectRepository, from: &Path, to: &Path) -> Result<(), UseCaseError> {
    for mut object in objects.all()? {
        let Some(rebased) = object.base().path.rebase(from, to) else {
            continue;
        };
        object.base_mut().path = rebased;
        objects.put(&object)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::prelude::*;
    use crate::testing::{MemoryStore, oid};
    use dam_domain::Task;

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
    fn children_are_the_objects_in_the_folder_an_object_owns() {
        let store = MemoryStore::new();
        let plan = put(&store, 1, "plan", "work");
        let step = put(&store, 2, "step", "work/plan");
        put(&store, 3, "detail", "work/plan/step");
        put(&store, 4, "note", "work");
        let owner = store.get(&plan).unwrap().unwrap();
        let found: Vec<Oid> = children(&store, owner.base())
            .unwrap()
            .iter()
            .map(|o| o.oid().clone())
            .collect();
        assert_eq!(found, vec![step]);
    }

    #[test]
    fn moving_an_object_into_a_folder_carries_the_folder_it_owns() {
        let store = MemoryStore::new();
        let step = put(&store, 2, "step", "work/plan");
        let detail = put(&store, 3, "detail", "work/plan/step");
        move_into(&store, &step, &Path::parse("work").unwrap()).unwrap();
        assert_eq!(path_of(&store, &step), "work/");
        assert_eq!(path_of(&store, &detail), "work/step/");
    }
}
