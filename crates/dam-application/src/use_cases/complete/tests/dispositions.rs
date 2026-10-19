use super::super::*;
use super::{path_of, put_task, today};
use crate::service::Dam;
use crate::testing::prelude::*;
use crate::testing::{FaultyStore, FixedRandom, Fixture, MemoryStore, oid};

fn with(children: ChildDisposition) -> Force {
    Force::With(Dispositions {
        children,
        dependencies: DependencyDisposition::Keep,
    })
}

#[test]
fn interactive_up_moves_children_beside_the_parent() {
    let store = MemoryStore::new();
    let plan = put_task(&store, 1, "plan", "work", false);
    let step = put_task(&store, 2, "step", "work/plan", false);
    let fixture = Fixture {
        random: FixedRandom::new(9),
        ..Fixture::default()
    };
    let dam = fixture.dam(&store);
    dam.complete(&plan, with(ChildDisposition::Up), Tracked::No)
        .unwrap();
    assert_eq!(path_of(&store, &step), "work/");
}

#[test]
fn interactive_into_groups_children_under_a_new_task_beside_the_parent() {
    let store = MemoryStore::new();
    let plan = put_task(&store, 1, "plan", "work", false);
    let step = put_task(&store, 2, "step", "work/plan", false);
    let fixture = Fixture {
        random: FixedRandom::new(9),
        ..Fixture::default()
    };
    let dam = fixture.dam(&store);
    dam.complete(
        &plan,
        with(ChildDisposition::Into("leftovers".into())),
        Tracked::No,
    )
    .unwrap();
    let group = store.get(&oid(9)).unwrap().unwrap();
    assert_eq!(group.base().subject, "leftovers");
    assert_eq!(group.base().path.as_str(), "work/");
    assert_eq!(path_of(&store, &step), "work/leftovers/");
}

#[test]
fn interactive_up_carries_a_grandchild_along() {
    let store = MemoryStore::new();
    let plan = put_task(&store, 1, "plan", "work", false);
    let step = put_task(&store, 2, "step", "work/plan", false);
    let detail = put_task(&store, 4, "detail", "work/plan/step", false);
    let fixture = Fixture {
        random: FixedRandom::new(9),
        ..Fixture::default()
    };
    let dam = fixture.dam(&store);
    dam.complete(&plan, with(ChildDisposition::Up), Tracked::No)
        .unwrap();
    assert_eq!(path_of(&store, &step), "work/");
    assert_eq!(path_of(&store, &detail), "work/step/");
}

#[test]
fn interactive_into_carries_a_grandchild_along() {
    let store = MemoryStore::new();
    let plan = put_task(&store, 1, "plan", "work", false);
    let step = put_task(&store, 2, "step", "work/plan", false);
    let detail = put_task(&store, 4, "detail", "work/plan/step", false);
    let fixture = Fixture {
        random: FixedRandom::new(9),
        ..Fixture::default()
    };
    let dam = fixture.dam(&store);
    dam.complete(
        &plan,
        with(ChildDisposition::Into("later".into())),
        Tracked::No,
    )
    .unwrap();
    assert_eq!(path_of(&store, &step), "work/later/");
    assert_eq!(path_of(&store, &detail), "work/later/step/");
}

#[test]
fn interactive_into_with_no_open_children_creates_no_group() {
    let store = MemoryStore::new();
    let id = put_task(&store, 1, "milk", "", false);
    let dep = put_task(&store, 3, "blocker", "", false);
    let mut t = store.get(&id).unwrap().unwrap();
    t.base_mut().depends.push(dep);
    store.put(&t).unwrap();
    let fixture = Fixture {
        random: FixedRandom::new(9),
        ..Fixture::default()
    };
    let dam = fixture.dam(&store);
    dam.complete(
        &id,
        with(ChildDisposition::Into("later".into())),
        Tracked::No,
    )
    .unwrap();
    assert!(store.get(&oid(9)).unwrap().is_none());
}

#[test]
fn interactive_drop_removes_open_dependencies() {
    let store = MemoryStore::new();
    let id = put_task(&store, 1, "milk", "", false);
    let dep = put_task(&store, 3, "blocker", "", false);
    let mut t = store.get(&id).unwrap().unwrap();
    t.base_mut().depends.push(dep);
    store.put(&t).unwrap();
    let d = Dispositions {
        children: ChildDisposition::Keep,
        dependencies: DependencyDisposition::Drop,
    };
    let fixture = Fixture {
        random: FixedRandom::new(9),
        ..Fixture::default()
    };
    let dam = fixture.dam(&store);
    dam.complete(&id, Force::With(d), Tracked::No).unwrap();
    let t = store.get(&id).unwrap().unwrap();
    assert!(t.base().depends.is_empty() && t.as_task().unwrap().completion.is_done());
}

#[test]
fn done_up_that_fails_between_two_children_moves_neither() {
    let store = FaultyStore::new(MemoryStore::new());
    let plan = put_task(&store.inner, 1, "plan", "work", false);
    put_task(&store.inner, 2, "step", "work/plan", false);
    put_task(&store.inner, 3, "other step", "work/plan", false);
    let random = FixedRandom::new(9);
    let fixture = Fixture::default();
    let dam = Dam {
        random: &random,
        ..fixture.dam(&store)
    };
    store.assert_unchanged_when_any_write_fails(|| {
        Dam {
            clock: &today(),
            ..dam
        }
        .complete(&plan, with(ChildDisposition::Up), Tracked::No)
        .map(drop)
    });
}

#[test]
fn done_into_that_fails_after_creating_the_group_leaves_no_group_behind() {
    let store = FaultyStore::new(MemoryStore::new());
    let plan = put_task(&store.inner, 1, "plan", "work", false);
    put_task(&store.inner, 2, "step", "work/plan", false);
    let dep = put_task(&store.inner, 3, "blocker", "", false);
    let mut t = store.inner.get(&plan).unwrap().unwrap();
    t.base_mut().depends.push(dep);
    store.inner.put(&t).unwrap();
    let d = Dispositions {
        children: ChildDisposition::Into("leftovers".into()),
        dependencies: DependencyDisposition::Drop,
    };
    let random = FixedRandom::new(9);
    let fixture = Fixture::default();
    let dam = Dam {
        random: &random,
        ..fixture.dam(&store)
    };
    store.assert_unchanged_when_any_write_fails(|| {
        Dam {
            clock: &today(),
            ..dam
        }
        .complete(&plan, Force::With(d.clone()), Tracked::No)
        .map(drop)
    });
}
