use dam_domain::{Oid, Path};

use crate::errors::{Refusal, UseCaseError};
use crate::ports::ObjectRepository;
use crate::use_cases::atomically::atomically;
use crate::use_cases::load::load;
use crate::use_cases::subtree::move_into;

pub fn relocate(objects: &dyn ObjectRepository, oid: &Oid, to: &Path) -> Result<(), UseCaseError> {
    atomically(objects, || {
        let object = load(objects, oid)?;
        if object
            .base()
            .owned_folder()
            .is_some_and(|folder| to.is_within(&folder))
        {
            return Err(Refusal::MoveInsideItself(oid.clone()).into());
        }
        move_into(objects, oid, to)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::prelude::*;
    use crate::testing::{FaultyStore, MemoryStore, oid};
    use dam_domain::{Object, Task};

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

    fn mv(store: &MemoryStore, id: &Oid, to: &str) -> Result<(), UseCaseError> {
        relocate(store, id, &Path::parse(to).unwrap())
    }

    #[test]
    fn moving_a_parent_carries_its_descendants() {
        let store = MemoryStore::new();
        let plan = put(&store, 1, "plan", "a");
        let step = put(&store, 2, "step", "a/plan");
        let detail = put(&store, 3, "detail", "a/plan/step");
        mv(&store, &plan, "b").unwrap();
        assert_eq!(path_of(&store, &plan), "b/");
        assert_eq!(path_of(&store, &step), "b/plan/");
        assert_eq!(path_of(&store, &detail), "b/plan/step/");
    }

    #[test]
    fn moving_into_yourself_is_refused() {
        let store = MemoryStore::new();
        let plan = put(&store, 1, "plan", "a");
        let err = mv(&store, &plan, "a/plan/c").unwrap_err();
        assert_eq!(err, UseCaseError::Refused(Refusal::MoveInsideItself(plan)));
    }

    #[test]
    fn a_top_level_object_lands_exactly_at_the_destination() {
        let store = MemoryStore::new();
        let x = put(&store, 1, "x", "");
        mv(&store, &x, "work").unwrap();
        assert_eq!(path_of(&store, &x), "work/");
    }

    #[test]
    fn moving_a_top_level_object_leaves_the_rest_of_the_store_where_it_was() {
        let store = MemoryStore::new();
        let a = put(&store, 1, "a", "");
        let b = put(&store, 2, "b", "");
        let home = put(&store, 3, "note", "home");
        let work = put(&store, 4, "plan", "work");
        mv(&store, &a, "work").unwrap();
        assert_eq!(path_of(&store, &a), "work/");
        assert_eq!(path_of(&store, &b), "");
        assert_eq!(path_of(&store, &home), "home/");
        assert_eq!(path_of(&store, &work), "work/");
    }

    #[test]
    fn moving_a_top_level_object_takes_its_children_and_nothing_else() {
        let store = MemoryStore::new();
        let a = put(&store, 1, "a", "");
        let child = put(&store, 2, "child", "a");
        let b = put(&store, 3, "b", "");
        let other = put(&store, 4, "other", "x");
        mv(&store, &a, "work").unwrap();
        assert_eq!(path_of(&store, &a), "work/");
        assert_eq!(path_of(&store, &child), "work/a/");
        assert_eq!(path_of(&store, &b), "");
        assert_eq!(path_of(&store, &other), "x/");
    }

    #[test]
    fn moving_to_the_root_puts_the_object_and_its_children_at_the_top_level() {
        let store = MemoryStore::new();
        let plan = put(&store, 1, "plan", "work");
        let step = put(&store, 2, "step", "work/plan");
        mv(&store, &plan, "").unwrap();
        assert_eq!(path_of(&store, &plan), "");
        assert_eq!(path_of(&store, &step), "plan/");
    }

    #[test]
    fn mv_that_fails_between_parent_and_child_moves_neither() {
        let store = FaultyStore::new(MemoryStore::new());
        let p = put(&store.inner, 1, "p", "a");
        put(&store.inner, 2, "c", "a/p");
        put(&store.inner, 3, "d", "a/p");
        store.assert_unchanged_when_any_write_fails(|| {
            relocate(&store, &p, &Path::parse("b").unwrap())
        });
    }
}
