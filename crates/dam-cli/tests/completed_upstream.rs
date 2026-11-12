mod support;

use serde_json::json;
use support::sandbox::Sandbox;
use support::upstream::{answer_pull_with, pulls_asked_since, task};

fn subjects(sb: &Sandbox, query: &str) -> Vec<String> {
    let (ok, out, err) = sb.dam(&["ls", query, "--json", "--no-pull"]);
    assert!(ok, "{err}");
    let listed: serde_json::Value = serde_json::from_str(&out).unwrap();
    listed["objects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["subject"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn a_task_completed_in_the_app_is_found_by_completed_today_and_a_replay_changes_nothing() {
    let _guard = support::guard(
        "a_task_completed_in_the_app_is_found_by_completed_today_and_a_replay_changes_nothing",
    );
    let sb = Sandbox::new();
    answer_pull_with(
        &sb,
        json!({"objects": [task("i:1", "water the plants", None)], "removed": [], "sync": "t1"}),
    );
    assert!(sb.dam(&["pull"]).0);
    assert!(subjects(&sb, "completed:today").is_empty());

    let now = jiff::Timestamp::now().to_string();
    let completed = json!({"objects": [task("i:1", "water the plants", Some(&now))], "removed": [], "sync": "t2"});
    answer_pull_with(&sb, completed.clone());
    assert!(sb.dam(&["pull"]).0);
    assert_eq!(subjects(&sb, "completed:today"), vec!["water the plants"]);

    answer_pull_with(&sb, completed);
    let (ok, out, err) = sb.dam(&["pull", "--json"]);
    assert!(ok, "{err}");
    let report: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(report["remotes"][0]["unchanged"], 1, "{report}");
    assert_eq!(subjects(&sb, "completed:today"), vec!["water the plants"]);
    assert_eq!(
        pulls_asked_since(&sb),
        vec![json!(null), json!("t1"), json!("t2")]
    );
}

#[test]
fn a_task_deleted_in_the_app_is_noticed_as_removed() {
    let _guard = support::guard("a_task_deleted_in_the_app_is_noticed_as_removed");
    let sb = Sandbox::new();
    answer_pull_with(
        &sb,
        json!({"objects": [task("i:1", "old errand", None)], "removed": [], "sync": "t1"}),
    );
    assert!(sb.dam(&["pull"]).0);
    answer_pull_with(
        &sb,
        json!({"objects": [], "removed": ["i:1"], "sync": "t2"}),
    );
    assert!(sb.dam(&["pull"]).0);
    let (ok, out, err) = sb.dam(&["status"]);
    assert!(ok, "{err}");
    assert!(out.contains("was removed on fake"), "{out}");
}
