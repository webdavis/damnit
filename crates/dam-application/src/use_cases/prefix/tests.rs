use dam_domain::{Object, Task};

use super::*;
use crate::testing::prelude::*;
use crate::testing::{MemoryStore, oid};

fn twin(last: u8) -> Oid {
    Oid::generate(&mut |x: &mut [u8]| {
        x.fill(0xab);
        x[19] = last;
    })
}

fn store_holding(oids: &[Oid]) -> MemoryStore {
    let store = MemoryStore::new();
    for (i, o) in oids.iter().enumerate() {
        store
            .put(&Object::Task(Task::new(o.clone(), format!("t{i}"))))
            .unwrap();
    }
    store
}

#[test]
fn a_full_oid_and_a_unique_prefix_resolve() {
    let store = store_holding(&[oid(0xab), oid(0xac)]);
    assert_eq!(resolve_object(&store, oid(0xab).as_str()), Ok(oid(0xab)));
    assert_eq!(resolve_object(&store, "abab"), Ok(oid(0xab)));
}

#[test]
fn a_full_oid_resolves_without_a_scan_so_one_only_history_holds_is_reached() {
    assert_eq!(
        resolve_object(&MemoryStore::new(), oid(0xcd).as_str()),
        Ok(oid(0xcd))
    );
}

#[test]
fn a_short_or_non_hex_prefix_is_not_an_oid() {
    let store = store_holding(&[twin(1)]);
    for text in ["ab", "zzzz", "f5"] {
        assert_eq!(
            resolve_object(&store, text),
            Err(UseCaseError::NotAnOid(text.into()))
        );
    }
    assert_eq!(
        UseCaseError::NotAnOid("f5".into()).to_string(),
        "\"f5\" is not an oid; give at least 4 hex characters"
    );
}

#[test]
fn a_prefix_naming_no_working_object_is_refused_by_the_layer_it_read() {
    let store = store_holding(&[twin(1)]);
    assert_eq!(
        resolve_object(&store, "cdcd"),
        Err(Refusal::NoWorkingObject("cdcd".into()).into())
    );
}

#[test]
fn a_prefix_naming_two_objects_lists_both() {
    let store = store_holding(&[twin(1), twin(2)]);
    assert_eq!(
        resolve_object(&store, "abab"),
        Err(Refusal::AmbiguousPrefix {
            text: "abab".into(),
            matches: vec![twin(1).to_string(), twin(2).to_string()],
        }
        .into())
    );
}
