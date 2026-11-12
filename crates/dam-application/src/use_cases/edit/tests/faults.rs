use super::super::*;
use crate::ports::StoreError;
use crate::testing::{FaultyStore, Scenario, oid, task, task_in};

#[test]
fn edit_that_fails_at_its_write_changes_nothing() {
    let s = Scenario::default();
    s.working(task(1));
    let id = oid(1);
    let store = FaultyStore::new(s.store);
    let dam = s.fixture.dam(&store);
    store.assert_unchanged_when_any_write_fails(|| {
        dam.edit(
            &id,
            EditFields {
                subject: Some("new".into()),
                ..EditFields::default()
            },
        )
        .map(drop)
    });
}

#[test]
fn a_rename_that_fails_between_parent_and_child_renames_and_moves_nothing() {
    let s = Scenario::default();
    s.filed(task_in(1, "p", "a"))
        .filed(task_in(2, "c", "a/p"))
        .filed(task_in(3, "d", "a/p/c"));
    let store = FaultyStore::new(s.store);
    let dam = s.fixture.dam(&store);
    store.assert_unchanged_when_any_write_fails(|| {
        dam.edit(
            &oid(1),
            EditFields {
                subject: Some("renamed".into()),
                ..EditFields::default()
            },
        )
        .map(drop)
    });
}

#[test]
fn a_dependency_that_cannot_be_read_fails_the_cycle_check() {
    let s = Scenario::default();
    s.working(task(1)).working(task(2));
    let (id, dependency) = (oid(1), oid(2));
    let store = FaultyStore::new(s.store);
    store.fail_reading(&dependency);
    let dam = s.fixture.dam(&store);
    let err = dam
        .edit(
            &id,
            EditFields {
                add_depends: vec![dependency],
                ..EditFields::default()
            },
        )
        .unwrap_err();
    assert_eq!(
        err,
        UseCaseError::Store(StoreError::Failed("unreadable object".into()))
    );
}
