use super::*;

#[test]
fn a_verdict_that_is_neither_ok_nor_an_error_object_is_a_failure() {
    let written = SyncWrite {
        verdict_by_uuid: HashMap::from([
            ("u-0".to_string(), serde_json::json!(true)),
            ("u-1".to_string(), serde_json::json!("done")),
        ]),
        ..SyncWrite::default()
    };
    assert!(written.accepted("u-0").is_err());
    assert!(written.accepted("u-1").is_err());
}
