use std::collections::BTreeSet;

use dam_domain::{Object, Task};

use super::super::reconcile::{Settlement, reconcile};
use crate::remote::{MutationOp, MutationOutcome, MutationVerdict, RemoteMutation};
use crate::testing::oid;

fn task(byte: u8) -> Object {
    Object::Task(Task::new(oid(byte), format!("t{byte}")))
}

fn create(byte: u8) -> RemoteMutation {
    RemoteMutation {
        op: MutationOp::Create,
        oid: oid(byte),
        idempotency_key: format!("k{byte}"),
        remote_id: None,
        object: Some(task(byte)),
        fields: vec![],
    }
}

fn accepted(byte: u8, remote_id: &str) -> MutationOutcome {
    MutationOutcome {
        oid: oid(byte),
        verdict: MutationVerdict::Accepted {
            remote_id: Some(remote_id.into()),
        },
    }
}

fn refused(byte: u8, why: &str) -> MutationOutcome {
    MutationOutcome {
        oid: oid(byte),
        verdict: MutationVerdict::Refused { why: why.into() },
    }
}

#[test]
fn only_the_first_answer_about_a_mutation_counts() {
    let settlement = reconcile(
        &[create(1)],
        vec![accepted(1, "r1"), refused(1, "too late")],
    );
    assert_eq!(
        settlement,
        Settlement {
            map: vec![(oid(1), "r1".into())],
            snapshot: vec![task(1)],
            ..Settlement::default()
        }
    );
}

#[test]
fn an_answer_about_a_mutation_never_sent_is_ignored() {
    let settlement = reconcile(&[create(1)], vec![accepted(1, "r1"), accepted(2, "r2")]);
    assert_eq!(
        settlement,
        Settlement {
            map: vec![(oid(1), "r1".into())],
            snapshot: vec![task(1)],
            ..Settlement::default()
        }
    );
}

#[test]
fn a_refused_mutation_fails_with_the_remote_reason() {
    let settlement = reconcile(&[create(1)], vec![refused(1, "rate limited")]);
    assert_eq!(
        settlement,
        Settlement {
            failed: vec![(oid(1), "rate limited".into())],
            ..Settlement::default()
        }
    );
}

#[test]
fn an_unanswered_mutation_fails_with_no_answer_from_the_helper() {
    let settlement = reconcile(&[create(1), create(2)], vec![accepted(1, "r1")]);
    assert_eq!(
        settlement,
        Settlement {
            map: vec![(oid(1), "r1".into())],
            snapshot: vec![task(1)],
            failed: vec![(oid(2), "no answer from helper".into())],
            unanswered: BTreeSet::from([oid(2)]),
            ..Settlement::default()
        }
    );
}

#[test]
fn an_accepted_delete_clears_the_mapping_instead_of_snapshotting() {
    let delete = RemoteMutation {
        op: MutationOp::Delete,
        remote_id: Some("r1".into()),
        object: None,
        ..create(1)
    };
    let settlement = reconcile(&[delete], vec![accepted(1, "r1")]);
    assert_eq!(
        settlement,
        Settlement {
            map: vec![(oid(1), "r1".into())],
            clear: vec![oid(1)],
            ..Settlement::default()
        }
    );
}
