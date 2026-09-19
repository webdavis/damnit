use dam_domain::{Oid, Path};

use crate::errors::{Refusal, UseCaseError};
use crate::ports::ObjectStore;
use crate::use_cases::subtree::{last_segment, move_subtree};

/// Moves an object to sit under `to`, carrying every descendant with it.
pub fn relocate(store: &dyn ObjectStore, oid: &Oid, to: &Path) -> Result<(), UseCaseError> {
    let object = store
        .get(oid)?
        .ok_or_else(|| Refusal::NoSuchObject(oid.short().to_string()))?;
    let old = object.base().path.clone();
    if to.is_within(&old) {
        return Err(UseCaseError::Parse(format!(
            "{} cannot move inside itself",
            oid.short()
        )));
    }
    let new = to
        .join(&last_segment(&old))
        .map_err(|e| UseCaseError::Parse(e.to_string()))?;
    move_subtree(store, oid, &new)
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
    fn moving_a_parent_carries_its_descendants() {
        let store = MemoryStore::new();
        let p = put(&store, 1, "a/p");
        let c = put(&store, 2, "a/p/c");
        let g = put(&store, 3, "a/p/c/g");
        relocate(&store, &p, &Path::parse("b").unwrap()).unwrap();
        assert_eq!(store.get(&p).unwrap().unwrap().base().path.as_str(), "b/p/");
        assert_eq!(
            store.get(&c).unwrap().unwrap().base().path.as_str(),
            "b/p/c/"
        );
        assert_eq!(
            store.get(&g).unwrap().unwrap().base().path.as_str(),
            "b/p/c/g/"
        );
    }

    #[test]
    fn moving_into_yourself_is_refused() {
        let store = MemoryStore::new();
        let p = put(&store, 1, "a/p");
        let err = relocate(&store, &p, &Path::parse("a/p/c").unwrap()).unwrap_err();
        assert!(matches!(err, UseCaseError::Parse(_)));
    }
}
