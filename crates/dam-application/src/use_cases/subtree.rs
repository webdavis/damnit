use dam_domain::{Oid, Path};

use crate::errors::{Refusal, UseCaseError};
use crate::ports::ObjectStore;

/// Moves `oid` to `to`, carrying every object nested under its old path along
/// with it, so a moved task's children (and their own children) relocate
/// instead of being orphaned under a stale prefix.
pub(crate) fn move_subtree(
    store: &dyn ObjectStore,
    oid: &Oid,
    to: &Path,
) -> Result<(), UseCaseError> {
    let mut object = store
        .get(oid)?
        .ok_or_else(|| Refusal::NoSuchObject(oid.short().to_string()))?;
    let from = object.base().path.clone();
    for mut other in store.all()? {
        if other.oid() == oid || !other.base().path.is_within(&from) {
            continue;
        }
        let suffix = &other.base().path.as_str()[from.as_str().len()..];
        other.base_mut().path = Path::parse(&format!("{}{suffix}", to.as_str()))
            .map_err(|e| UseCaseError::Parse(e.to_string()))?;
        store.put(&other)?;
    }
    object.base_mut().path = to.clone();
    store.put(&object)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{MemoryStore, oid};
    use dam_domain::{Object, Task};

    fn put(store: &MemoryStore, byte: u8, path: &str) -> Oid {
        let mut t = Task::new(oid(byte), format!("t{byte}"));
        t.base.path = Path::parse(path).unwrap();
        store.put(&Object::Task(t)).unwrap();
        oid(byte)
    }

    #[test]
    fn moving_an_object_carries_its_descendants() {
        let store = MemoryStore::new();
        let c = put(&store, 2, "p/c");
        let g = put(&store, 3, "p/c/g");
        move_subtree(&store, &c, &Path::parse("c").unwrap()).unwrap();
        assert_eq!(store.get(&c).unwrap().unwrap().base().path.as_str(), "c/");
        assert_eq!(store.get(&g).unwrap().unwrap().base().path.as_str(), "c/g/");
    }

    #[test]
    fn moving_a_leaf_touches_only_itself() {
        let store = MemoryStore::new();
        let c = put(&store, 2, "p/c");
        move_subtree(&store, &c, &Path::parse("elsewhere").unwrap()).unwrap();
        assert_eq!(
            store.get(&c).unwrap().unwrap().base().path.as_str(),
            "elsewhere/"
        );
    }
}
