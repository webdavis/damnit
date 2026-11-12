mod support;

use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to, refused};

fn committed_task(sb: &Sandbox, subject: &str) -> String {
    let oid = sb.new_object(&[subject]);
    for line in [vec!["add", "-A"], vec!["commit", "-m", "seed"]] {
        let (ok, _, err) = sb.dam(&line);
        assert!(ok, "{err}");
    }
    oid
}

fn shown(sb: &Sandbox, oid: &str) -> serde_json::Value {
    let (ok, out, err) = sb.dam(&["show", oid, "--json"]);
    assert!(ok, "{err}");
    serde_json::from_str(&out).unwrap()
}

#[test]
fn a_tokens_estimate_waits_as_an_item_the_owner_adds() {
    let _guard = support::guard("a_tokens_estimate_waits_as_an_item_the_owner_adds");
    let sb = Sandbox::new();
    let oid = committed_task(&sb, "File taxes");
    let held = grant_to(&sb, "vpt", 1, &grant("\"task:read\", \"task:change\""));
    let line = ["edit", &oid, "--estimate", "30", "--json"];
    let sent = answer(&sb.output_holding(&held, &line), "sent.schema.json");
    assert_eq!(sent["answer"], "changed");
    assert_eq!(
        shown(&sb, &oid)["task"]["estimate"],
        serde_json::Value::Null
    );

    let (ok, _, err) = sb.dam(&["add", sent["hash"].as_str().unwrap()]);
    assert!(ok, "{err}");
    assert_eq!(shown(&sb, &oid)["task"]["estimate"], 30);
}

#[test]
fn a_token_cannot_mark_a_task_as_the_owners_focus() {
    let _guard = support::guard("a_token_cannot_mark_a_task_as_the_owners_focus");
    let sb = Sandbox::new();
    let oid = committed_task(&sb, "File taxes");
    let held = grant_to(&sb, "vpt", 1, &grant("\"task:read\", \"task:change\""));
    let out = sb.output_holding(&held, &["edit", &oid, "--focus", "today", "--json"]);
    assert_eq!(refused(&out), "not_open_to_tokens");
    let (ok, status, err) = sb.dam(&["status", "--json"]);
    assert!(ok, "{err}");
    let status: serde_json::Value = serde_json::from_str(&status).unwrap();
    assert_eq!(status["waiting"], serde_json::json!([]));
}
