use crate::support;

use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;

#[test]
fn hook_run_calls_a_real_hook_file_and_leaves_its_change_as_a_working_change_the_log_names() {
    let _guard = support::guard(
        "hook_run_calls_a_real_hook_file_and_leaves_its_change_as_a_working_change_the_log_names",
    );
    let sb = Sandbox::new();
    let (ok, _, err) = sb.dam(&["new", "buy milk"]);
    assert!(ok, "{err}");
    sb.write_hooks(&[(
        "task_changed",
        "function(task) return { subject = task.subject .. ' today' } end",
    )]);
    let out = sb.output(&["hook", "run", "task_changed", "-o", "json"]);
    assert!(out.status.success(), "{out:?}");
    let answer: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_matches_schema(&answer, "hook-run.schema.json");
    assert_eq!(
        (&answer["ran"], &answer["changed"], &answer["error"]),
        (&1.into(), &1.into(), &serde_json::Value::Null)
    );
    let (ok, out, err) = sb.dam(&["add", "-A"]);
    assert!(ok, "{err}{out}");
    let (ok, _, err) = sb.dam(&["commit", "-m", "hooked"]);
    assert!(ok, "{err}");
    let (ok, out, err) = sb.dam(&["log", "-o", "json"]);
    assert!(ok, "{err}");
    assert!(out.contains("buy milk today"), "{out}");
    let log: serde_json::Value = serde_json::from_str(&out).unwrap();
    let hooks: Vec<&serde_json::Value> = log["commits"][0]["changes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|change| &change["details"]["meta"]["hook"])
        .collect();
    assert_eq!(hooks, [&serde_json::json!("task_changed")], "{out}");
}
