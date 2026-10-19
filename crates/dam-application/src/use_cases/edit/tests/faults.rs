use super::super::*;
use super::seed;
use crate::ports::StoreError;
use crate::testing::{FaultyStore, MemoryStore};
use dam_domain::Categories;

#[test]
fn edit_that_fails_at_its_write_changes_nothing() {
    let store = FaultyStore::new(MemoryStore::new());
    let id = seed(&store.inner, 1);
    store.assert_unchanged_when_any_write_fails(|| {
        edit(
            &store,
            &Categories::default(),
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
fn a_dependency_that_cannot_be_read_fails_the_cycle_check() {
    let store = FaultyStore::new(MemoryStore::new());
    let id = seed(&store.inner, 1);
    let dependency = seed(&store.inner, 2);
    store.fail_reading(&dependency);
    let err = edit(
        &store,
        &Categories::default(),
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
