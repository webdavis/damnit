use super::super::*;
use super::seed;
use crate::ports::StoreError;
use crate::testing::{FaultyStore, Fixture, MemoryStore};

#[test]
fn edit_that_fails_at_its_write_changes_nothing() {
    let store = FaultyStore::new(MemoryStore::new());
    let id = seed(&store.inner, 1);
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
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
fn a_dependency_that_cannot_be_read_fails_the_cycle_check() {
    let store = FaultyStore::new(MemoryStore::new());
    let id = seed(&store.inner, 1);
    let dependency = seed(&store.inner, 2);
    store.fail_reading(&dependency);
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
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
