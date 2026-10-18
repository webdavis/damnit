mod support;

use support::sandbox::Sandbox;

const DONE_UPSTREAM: &str = r#"{"oid":"","remote_id":"up-1","kind":"task","subject":"upstream","body":"","path":"","labels":[],"depends":[],"reminders":[],"recurrence":null,"task":{"done":true,"completed_at":"2026-09-18T15:04:05Z","priority":2,"due":null,"deadline":null,"event":null},"event":null}"#;

const STORED_TASK: &str =
    r#""task":{"done":true,"completed_at":"2026-09-18T15:04:05Z","priority":2}"#;

#[test]
fn a_done_task_keeps_done_and_completed_at_as_two_keys_in_the_store_and_in_show() {
    let _guard = support::guard(
        "a_done_task_keeps_done_and_completed_at_as_two_keys_in_the_store_and_in_show",
    );
    let sb = Sandbox::new();
    sb.install_helper_pulling(DONE_UPSTREAM);
    let (ok, _, err) = sb.dam(&["pull"]);
    assert!(ok, "{err}");
    let (ok, out, err) = sb.dam(&["ls", "--json", "--no-pull"]);
    assert!(ok, "{err}");
    let listed: serde_json::Value = serde_json::from_str(&out).unwrap();
    let oid = listed["objects"][0]["oid"].as_str().unwrap().to_string();

    let (ok, out, err) = sb.dam(&["show", &oid, "--json", "--no-pull"]);
    assert!(ok, "{err}");
    let shown: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        shown["task"],
        serde_json::json!({"done": true, "completed_at": "2026-09-18T15:04:05Z", "priority": 2}),
        "{shown}"
    );
    let stored = sb.store_bytes();
    assert!(
        stored.contains(STORED_TASK),
        "stored task is not {STORED_TASK}"
    );
}
