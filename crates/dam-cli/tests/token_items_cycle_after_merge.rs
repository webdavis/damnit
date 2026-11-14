mod support;

use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to, refused};

fn dam(sb: &Sandbox, line: &[&str]) {
    let (ok, _, err) = sb.dam(line);
    assert!(ok, "{err}");
}

#[test]
fn taking_a_tokens_side_refuses_a_dependency_the_owner_dropped_that_now_closes_a_cycle() {
    let _guard = support::guard(
        "taking_a_tokens_side_refuses_a_dependency_the_owner_dropped_that_now_closes_a_cycle",
    );
    let sb = Sandbox::new();
    let rent = sb.new_object(&["Pay rent"]);
    let keys = sb.new_object(&["Find the keys"]);
    let desk = sb.new_object(&["Clear the desk"]);
    dam(&sb, &["edit", &rent, "--depends", &keys]);
    dam(&sb, &["add", "-A"]);
    dam(&sb, &["commit", "-m", "rent waits on the keys"]);
    let held = grant_to(&sb, "tool", 1, vec![grant(&["task:read", "task:change"])]);
    let line = ["edit", &rent, "--depends", &desk, "--json"];
    let sent = answer(&sb.output_holding(&held, &line), "sent.schema.json");
    let hash = sent["hash"].as_str().unwrap();
    dam(&sb, &["edit", &rent, "--undepends", &keys]);
    dam(&sb, &["edit", &keys, "--depends", &rent]);

    let out = sb.output(&["resolve", hash, "--theirs", "--json"]);
    assert_eq!(refused(&out), "item_cycle");
    let (_, status, _) = sb.dam(&["status", "--json"]);
    let status: serde_json::Value = serde_json::from_str(&status).unwrap();
    assert_eq!(status["waiting"][0]["hash"], hash);
}
