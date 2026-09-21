use dam_domain::{Oid, Path};

use crate::errors::{Refusal, UseCaseError};
use crate::ports::ObjectRepository;
use crate::use_cases::subtree::{last_segment, move_subtree};

/// Moves an object to sit under `to`, carrying every descendant with it.
pub fn relocate(objects: &dyn ObjectRepository, oid: &Oid, to: &Path) -> Result<(), UseCaseError> {
    let object = objects
        .get(oid)?
        .ok_or_else(|| Refusal::NoSuchObject(oid.short().to_string()))?;
    let old = object.base().path.clone();
    // A root path ("") is shared by every top-level object, so it never
    // meaningfully contains anything: only a real, owned prefix can.
    if !old.as_str().is_empty() && to.is_within(&old) {
        return Err(Refusal::MoveInsideItself(oid.clone()).into());
    }
    let new = if old.as_str().is_empty() {
        to.clone()
    } else {
        to.join(&last_segment(&old))
            .map_err(|e| UseCaseError::Parse(e.to_string()))?
    };
    move_subtree(objects, oid, &new)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::prelude::*;
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
        assert_eq!(err, UseCaseError::Refused(Refusal::MoveInsideItself(p)));
    }

    #[test]
    fn a_top_level_object_lands_exactly_at_the_destination() {
        let store = MemoryStore::new();
        let x = put(&store, 1, "");
        relocate(&store, &x, &Path::parse("work").unwrap()).unwrap();
        assert_eq!(
            store.get(&x).unwrap().unwrap().base().path.as_str(),
            "work/"
        );
    }
}
