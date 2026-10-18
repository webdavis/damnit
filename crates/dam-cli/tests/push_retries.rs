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

#[test]
fn a_retry_keeps_the_failed_field_when_a_later_commit_changes_another() {
    let _guard =
        support::guard("a_retry_keeps_the_failed_field_when_a_later_commit_changes_another");
    let sb = Sandbox::new();
    let oid = sb.new_object(&["buy oat milk"]);
    assert!(sb.dam(&["add", "-A"]).0);
    assert!(sb.dam(&["commit", "-m", "create"]).0);
    assert!(sb.dam(&["push"]).0);

    assert!(sb.dam(&["edit", &oid, "--subject", "buy soy milk"]).0);
    assert!(sb.dam(&["add", "-A"]).0);
    assert!(sb.dam(&["commit", "-m", "rename"]).0);
    sb.fail_next_push();
    let (_, out, _) = sb.dam(&["push"]);
    assert!(out.contains("fake: 1 sent, 0 ok, 1 failed"), "{out}");

    assert!(sb.dam(&["done", &oid]).0);
    assert!(sb.dam(&["add", "-A"]).0);
    assert!(sb.dam(&["commit", "-m", "done"]).0);
    let (ok, out, err) = sb.dam(&["push"]);
    assert!(ok, "{err}");
    assert!(out.contains("fake: 1 sent, 1 ok"), "{out}");
    let retry = sb.pushes().pop().unwrap();
    assert_eq!(
        retry["mutations"][0]["fields"],
        serde_json::json!(["subject", "done"])
    );

    let (ok, _, err) = sb.dam(&["pull"]);
    assert!(ok, "{err}");
    let (ok, out, _) = sb.dam(&["show", &oid]);
    assert!(ok);
    assert!(out.contains("buy soy milk"), "{out}");
}
