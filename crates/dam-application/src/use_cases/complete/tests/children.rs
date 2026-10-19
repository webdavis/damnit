use super::super::*;
use super::{path_of, put_task, today};
use crate::testing::prelude::*;
use crate::testing::{FixedRandom, MemoryStore};
use dam_domain::Blockers;

#[test]
fn a_root_task_with_objects_in_subfolders_has_no_children() {
    let store = MemoryStore::new();
    let milk = put_task(&store, 1, "buy oat milk", "", false);
    put_task(&store, 2, "standup notes", "work", false);
    put_task(&store, 3, "another", "work/clients/acme", false);
    assert!(plan_complete(&store, &milk).unwrap().blockers.is_empty());
    assert_eq!(
        complete(
            &store,
            &today(),
            &FixedRandom::new(9),
            &milk,
            Force::No,
            Tracked::No
        )
        .unwrap(),
        Completed::Done
    );
}

#[test]
fn objects_in_a_siblings_folder_are_not_children() {
    let store = MemoryStore::new();
    let offsite = put_task(&store, 1, "plan the offsite", "work", false);
    put_task(&store, 2, "unrelated project note", "work/clients", false);
    put_task(&store, 3, "another", "work/clients/acme", false);
    assert!(plan_complete(&store, &offsite).unwrap().blockers.is_empty());
}

#[test]
fn open_children_and_dependencies_block_and_force_no_refuses() {
    let store = MemoryStore::new();
    let plan = put_task(&store, 1, "plan", "work", false);
    let step = put_task(&store, 2, "step", "work/plan", false);
    let dep = put_task(&store, 3, "blocker", "", false);
    let mut p = store.get(&plan).unwrap().unwrap();
    p.base_mut().depends.push(dep.clone());
    store.put(&p).unwrap();
    let found = plan_complete(&store, &plan).unwrap();
    assert_eq!(
        found.blockers,
        Blockers {
            dependencies: vec![dep.clone()],
            children: vec![step.clone()],
        }
    );
    let err = complete(
        &store,
        &today(),
        &FixedRandom::new(9),
        &plan,
        Force::No,
        Tracked::No,
    )
    .unwrap_err();
    assert!(matches!(
        err,
        UseCaseError::Refused(Refusal::Blocked { .. })
    ));
    assert!(
        !store
            .get(&plan)
            .unwrap()
            .unwrap()
            .as_task()
            .unwrap()
            .completion
            .is_done()
    );
}

#[test]
fn done_children_do_not_block() {
    let store = MemoryStore::new();
    let plan = put_task(&store, 1, "plan", "work", false);
    put_task(&store, 2, "step", "work/plan", true);
    assert!(plan_complete(&store, &plan).unwrap().blockers.is_empty());
}

#[test]
fn forcing_with_default_dispositions_completes_and_leaves_children_where_they_are() {
    let store = MemoryStore::new();
    let plan = put_task(&store, 1, "plan", "work", false);
    let step = put_task(&store, 2, "step", "work/plan", false);
    complete(
        &store,
        &today(),
        &FixedRandom::new(9),
        &plan,
        Force::With(Dispositions::default()),
        Tracked::No,
    )
    .unwrap();
    assert!(
        store
            .get(&plan)
            .unwrap()
            .unwrap()
            .as_task()
            .unwrap()
            .completion
            .is_done()
    );
    assert_eq!(path_of(&store, &step), "work/plan/");
}
