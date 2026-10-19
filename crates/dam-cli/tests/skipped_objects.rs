mod support;

use support::sandbox::Sandbox;

const LOOPING_UP_7: &str = r#"{"objects":[],"rejected":[{"remote_id":"up-7","why":"its parent chain loops"}],"removed":[],"sync":null}"#;

#[test]
fn an_object_a_pull_skipped_reads_as_skipped_not_as_a_failed_pull() {
    let _guard = support::guard("an_object_a_pull_skipped_reads_as_skipped_not_as_a_failed_pull");
    let sb = Sandbox::new();
    sb.install_helper_answering_pull(LOOPING_UP_7);
    let (ok, _, err) = sb.dam(&["pull"]);
    assert!(ok, "{err}");

    let (ok, out, err) = sb.dam(&["status"]);
    assert!(ok, "{err}");
    assert!(
        out.contains("pull from fake skipped up-7: its parent chain loops"),
        "{out}"
    );
    assert!(!out.contains("failed"), "{out}");

    let (ok, out, err) = sb.dam(&["status", "--json"]);
    assert!(ok, "{err}");
    let status: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        status["notices"],
        serde_json::json!([{
            "kind": "pull_skipped",
            "remote": "fake",
            "remote_id": "up-7",
            "why": "its parent chain loops"
        }])
    );
}
