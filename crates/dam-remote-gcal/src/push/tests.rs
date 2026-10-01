use super::*;

fn mutation(oid: &str, op: &str) -> Mutation {
    Mutation {
        op: op.into(),
        oid: oid.into(),
        idempotency_key: "k".into(),
        remote_id: None,
        object: None,
        fields: vec![],
    }
}

#[test]
fn every_mutation_is_refused_with_the_sentence_in_order() {
    let answer = refuse(&[mutation("a", "create"), mutation("b", "delete")]);
    assert_eq!(
        answer
            .results
            .iter()
            .map(|r| r.oid.as_str())
            .collect::<Vec<_>>(),
        vec!["a", "b"]
    );
    assert!(
        answer
            .results
            .iter()
            .all(|r| !r.ok && r.why.as_deref() == Some(READ_ONLY) && r.remote_id.is_none())
    );
    assert!(READ_ONLY.contains("read-only"));
}
