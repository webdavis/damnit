use super::super::*;
use super::{put_task, today};
use crate::testing::prelude::*;
use crate::testing::{FixedRandom, MemoryStore, oid};

#[test]
fn interactive_up_moves_children_beside_the_parent() {
    let store = MemoryStore::new();
    let parent = put_task(&store, 1, "top/p", false);
    let child = put_task(&store, 2, "top/p/c", false);
    let d = Dispositions {
        children: ChildDisposition::Up,
        dependencies: DependencyDisposition::Keep,
    };
    complete(
        &store,
        &today(),
        &FixedRandom::new(9),
        &parent,
        Force::With(d),
    )
    .unwrap();
    assert_eq!(
        store.get(&child).unwrap().unwrap().base().path.as_str(),
        "top/c/"
    );
}

#[test]
fn interactive_into_groups_children_under_a_new_task() {
    let store = MemoryStore::new();
    let parent = put_task(&store, 1, "p", false);
    let child = put_task(&store, 2, "p/c", false);
    let d = Dispositions {
        children: ChildDisposition::Into("leftovers".into()),
        dependencies: DependencyDisposition::Keep,
    };
    complete(
        &store,
        &today(),
        &FixedRandom::new(9),
        &parent,
        Force::With(d),
    )
    .unwrap();
    let group = store.get(&oid(9)).unwrap().unwrap();
    assert_eq!(group.base().subject, "leftovers");
    assert_eq!(group.base().path.as_str(), "leftovers/");
    assert_eq!(
        store.get(&child).unwrap().unwrap().base().path.as_str(),
        "leftovers/c/"
    );
}

#[test]
fn interactive_up_carries_a_grandchild_along() {
    let store = MemoryStore::new();
    let parent = put_task(&store, 1, "p", false);
    let child = put_task(&store, 2, "p/c", false);
    let grandchild = put_task(&store, 4, "p/c/g", false);
    let d = Dispositions {
        children: ChildDisposition::Up,
        dependencies: DependencyDisposition::Keep,
    };
    complete(
        &store,
        &today(),
        &FixedRandom::new(9),
        &parent,
        Force::With(d),
    )
    .unwrap();
    assert_eq!(
        store.get(&child).unwrap().unwrap().base().path.as_str(),
        "c/"
    );
    assert_eq!(
        store
            .get(&grandchild)
            .unwrap()
            .unwrap()
            .base()
            .path
            .as_str(),
        "c/g/"
    );
}

#[test]
fn interactive_into_carries_a_grandchild_along() {
    let store = MemoryStore::new();
    let parent = put_task(&store, 1, "p", false);
    let child = put_task(&store, 2, "p/c", false);
    let grandchild = put_task(&store, 4, "p/c/g", false);
    let d = Dispositions {
        children: ChildDisposition::Into("later".into()),
        dependencies: DependencyDisposition::Keep,
    };
    complete(
        &store,
        &today(),
        &FixedRandom::new(9),
        &parent,
        Force::With(d),
    )
    .unwrap();
    assert_eq!(
        store.get(&child).unwrap().unwrap().base().path.as_str(),
        "later/c/"
    );
    assert_eq!(
        store
            .get(&grandchild)
            .unwrap()
            .unwrap()
            .base()
            .path
            .as_str(),
        "later/c/g/"
    );
}

#[test]
fn interactive_into_with_no_open_children_creates_no_group() {
    let store = MemoryStore::new();
    let id = put_task(&store, 1, "", false);
    let dep = put_task(&store, 3, "", false);
    let mut t = store.get(&id).unwrap().unwrap();
    t.base_mut().depends.push(dep);
    store.put(&t).unwrap();
    let d = Dispositions {
        children: ChildDisposition::Into("later".into()),
        dependencies: DependencyDisposition::Keep,
    };
    complete(&store, &today(), &FixedRandom::new(9), &id, Force::With(d)).unwrap();
    assert!(store.get(&oid(9)).unwrap().is_none());
}

#[test]
fn interactive_drop_removes_open_dependencies() {
    let store = MemoryStore::new();
    let id = put_task(&store, 1, "", false);
    let dep = put_task(&store, 3, "", false);
    let mut t = store.get(&id).unwrap().unwrap();
    t.base_mut().depends.push(dep);
    store.put(&t).unwrap();
    let d = Dispositions {
        children: ChildDisposition::Keep,
        dependencies: DependencyDisposition::Drop,
    };
    complete(&store, &today(), &FixedRandom::new(9), &id, Force::With(d)).unwrap();
    let t = store.get(&id).unwrap().unwrap();
    assert!(t.base().depends.is_empty() && t.as_task().unwrap().done);
}
