mod support;

use support::sandbox::Sandbox;

#[test]
fn a_retry_after_a_restart_declares_only_the_fields_of_the_change_that_failed() {
    let _guard = support::guard(
        "a_retry_after_a_restart_declares_only_the_fields_of_the_change_that_failed",
    );
    let sb = Sandbox::new();
    let oid = sb.new_object(&["buy oat milk"]);
    assert!(sb.dam(&["add", "-A"]).0);
    assert!(sb.dam(&["commit", "-m", "create"]).0);
    let (ok, out, err) = sb.dam(&["push"]);
    assert!(ok, "{err}");
    assert!(out.contains("fake: 1 sent, 1 ok"), "{out}");

    assert!(sb.dam(&["edit", &oid, "--subject", "buy soy milk"]).0);
    assert!(sb.dam(&["add", "-A"]).0);
    assert!(sb.dam(&["commit", "-m", "rename"]).0);
    sb.fail_next_push();
    let (_, out, _) = sb.dam(&["push"]);
    assert!(out.contains("fake: 1 sent, 0 ok, 1 failed"), "{out}");

    let (ok, out, err) = sb.dam(&["push"]);
    assert!(ok, "{err}");
    assert!(out.contains("fake: 1 sent, 1 ok"), "{out}");
    let retry = sb.pushes().pop().unwrap();
    assert_eq!(retry["mutations"][0]["op"], "update");
    assert_eq!(
        retry["mutations"][0]["fields"],
        serde_json::json!(["subject"])
    );
}
