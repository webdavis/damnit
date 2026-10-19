mod support;

use support::sandbox::Sandbox;
use support::status_json;

#[test]
fn a_completion_time_appears_in_the_object_and_the_change_and_is_cleared_by_undone() {
    let _guard = support::guard(
        "a_completion_time_appears_in_the_object_and_the_change_and_is_cleared_by_undone",
    );
    let sb = Sandbox::new();
    let oid = sb.new_object(&["buy oat milk"]);

    let (ok, out, err) = sb.dam(&["show", &oid, "--json"]);
    assert!(ok, "{err}");
    let open: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        open["task"]["completed_at"],
        serde_json::Value::Null,
        "{open}"
    );

    let (ok, _, err) = sb.dam(&["done", &oid]);
    assert!(ok, "{err}");
    let (ok, out, err) = sb.dam(&["show", &oid, "--json"]);
    assert!(ok, "{err}");
    let done: serde_json::Value = serde_json::from_str(&out).unwrap();
    let at = done["task"]["completed_at"]
        .as_str()
        .unwrap_or_else(|| panic!("no completion time: {done}"));
    at.parse::<jiff::Timestamp>()
        .unwrap_or_else(|e| panic!("completed_at is not RFC 3339: {e} ({at})"));

    let row = status_json(&sb, &["--json"])["unstaged"][0].clone();
    assert_eq!(row["done"], true, "{row}");
    assert_eq!(row["completed_at"], at, "{row}");

    let (ok, _, err) = sb.dam(&["edit", &oid, "--undone"]);
    assert!(ok, "{err}");
    let (ok, out, err) = sb.dam(&["show", &oid, "--json"]);
    assert!(ok, "{err}");
    let reopened: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        reopened["task"]["completed_at"],
        serde_json::Value::Null,
        "{reopened}"
    );
}

#[test]
fn completing_a_task_twice_leaves_nothing_to_report() {
    let _guard = support::guard("completing_a_task_twice_leaves_nothing_to_report");
    let sb = Sandbox::new();
    let oid = sb.new_object(&["repeat done test"]);
    assert!(sb.dam(&["done", &oid]).0);
    assert!(sb.dam(&["add", "-A"]).0);
    assert!(sb.dam(&["commit", "-m", "done it"]).0);

    assert!(sb.dam(&["done", &oid]).0);
    assert_eq!(
        status_json(&sb, &["--json"])["unstaged"],
        serde_json::json!([]),
        "the second done made a change describing nothing"
    );
}

#[test]
fn a_helper_cannot_put_a_completion_time_on_an_open_task() {
    let _guard = support::guard("a_helper_cannot_put_a_completion_time_on_an_open_task");
    let sb = Sandbox::new();
    let task = r#"{"oid":"","remote_id":"up-1","kind":"task","subject":"upstream","body":"","path":"","labels":[],"depends":[],"reminders":[],"recurrence":null,"task":{"done":false,"completed_at":"2020-01-01T00:00:00Z","priority":2,"due":null,"deadline":null,"event":null},"event":null}"#;
    let helper = sb.dir.path().join("bin/dam-remote-fake");
    std::fs::write(
        &helper,
        format!(
            r#"#!/bin/sh
set -eu
while IFS= read -r line; do
  case "$line" in
    *'"capabilities"'*) printf '{{"protocol":1,"kinds":["task"],"fields":["subject"],"credentials":["api_token"],"incremental":false}}\n' ;;
    *'"pull"'*) printf '{{"objects":[{task}],"removed":[],"sync":null}}\n' ;;
    *) printf '{{"error":"unknown request"}}\n' ;;
  esac
done
"#
        ),
    )
    .unwrap();
    std::fs::set_permissions(&helper, std::os::unix::fs::PermissionsExt::from_mode(0o755)).unwrap();

    let (ok, _, err) = sb.dam(&["pull", "--json"]);
    assert!(ok, "{err}");
    let (ok, out, err) = sb.dam(&["ls", "--json", "--no-pull"]);
    assert!(ok, "{err}");
    let answer: serde_json::Value = serde_json::from_str(&out).unwrap();
    let task = &answer["objects"][0]["task"];
    assert_eq!(task["done"], false, "{answer}");
    assert_eq!(
        task["completed_at"],
        serde_json::Value::Null,
        "an open task was stored carrying a completion time: {answer}"
    );
}
