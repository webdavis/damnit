use std::collections::BTreeMap;

use dam_domain::Field;

use super::{oid, remote};
use crate::ports::{Attempt, AttemptRepository};
use crate::remote::MutationOp;

fn attempt(key: &str, op: MutationOp, declared: &[(Field, &str)]) -> Attempt {
    Attempt {
        key: key.into(),
        op,
        container_remote_id: None,
        declared: declared.iter().map(|(f, v)| (*f, v.to_string())).collect(),
    }
}

pub fn attempt_repository_contract(store: &dyn AttemptRepository) {
    let here = remote("here");
    let there = remote("there");
    assert!(store.attempts(&here).unwrap().is_empty());
    let create = attempt(
        "k1",
        MutationOp::Create,
        &[(Field::Subject, r#""Ada""#), (Field::Done, "false")],
    );
    let moved = Attempt {
        container_remote_id: Some("p1".into()),
        ..attempt("k2", MutationOp::Update, &[(Field::Path, r#""work/""#)])
    };
    let delete = attempt("k3", MutationOp::Delete, &[]);
    let in_order = vec![create.clone(), moved, delete];
    store.set_attempts(&here, &oid(1), &in_order).unwrap();
    store.set_attempts(&here, &oid(2), &[create]).unwrap();
    assert!(
        store.attempts(&there).unwrap().is_empty(),
        "attempts belong to one remote"
    );
    store.set_attempts(&here, &oid(2), &[]).unwrap();
    assert_eq!(
        store.attempts(&here).unwrap(),
        BTreeMap::from([(oid(1), in_order)]),
        "an object's attempts come back in order, and none leaves it out"
    );
}
