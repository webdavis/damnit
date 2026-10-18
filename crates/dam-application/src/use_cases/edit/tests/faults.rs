use super::super::*;
use super::seed;
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
