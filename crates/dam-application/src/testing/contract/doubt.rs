use std::collections::{BTreeMap, BTreeSet};

use dam_domain::Field;

use super::{oid, remote};
use crate::ports::DoubtRepository;

pub fn doubt_repository_contract(store: &dyn DoubtRepository) {
    let here = remote("here");
    let there = remote("there");
    assert!(store.fields_in_doubt(&here).unwrap().is_empty());
    let both = BTreeSet::from([Field::Subject, Field::Done]);
    store.set_fields_in_doubt(&here, &oid(1), &both).unwrap();
    store
        .set_fields_in_doubt(&here, &oid(2), &BTreeSet::from([Field::Due]))
        .unwrap();
    assert!(
        store.fields_in_doubt(&there).unwrap().is_empty(),
        "doubt belongs to one remote"
    );
    store
        .set_fields_in_doubt(&here, &oid(1), &BTreeSet::from([Field::Done]))
        .unwrap();
    store
        .set_fields_in_doubt(&here, &oid(2), &BTreeSet::new())
        .unwrap();
    assert_eq!(
        store.fields_in_doubt(&here).unwrap(),
        BTreeMap::from([(oid(1), BTreeSet::from([Field::Done]))]),
        "setting replaces an object's fields, and no fields leaves it out"
    );
}
