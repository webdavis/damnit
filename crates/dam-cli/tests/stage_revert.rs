mod support;

use support::sandbox::Sandbox;
use support::status_json;

#[test]
fn staging_an_object_edited_back_to_its_committed_state_leaves_nothing_staged() {
    let _guard = support::guard(
        "staging_an_object_edited_back_to_its_committed_state_leaves_nothing_staged",
    );
    let sb = Sandbox::new();
    let oid = sb.new_object(&["a"]);
    assert!(sb.dam(&["add", "-A"]).0);
    assert!(sb.dam(&["commit", "-m", "a"]).0);

    assert!(sb.dam(&["edit", &oid, "--subject", "b"]).0);
    assert!(sb.dam(&["add", "-A"]).0);
    assert!(sb.dam(&["edit", &oid, "--subject", "a"]).0);
    assert!(sb.dam(&["add", "-A"]).0);

    let status = status_json(&sb, &["--json"]);
    assert_eq!(status["staged"], serde_json::json!([]));
    assert_eq!(status["unstaged"], serde_json::json!([]));
}
