mod support;

use serde_json::json;
use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;
use support::status_json;
use support::upstream::{answer_pull_with, task};

fn pulled_by_a_failing_hook() -> Sandbox {
    let sb = Sandbox::new();
    sb.write_config_lua(
        "return {\n  remote = { fake = { url = 'fake::' } },\n  \
         hook = { task_pulled = function() error('no bell') end },\n}\n",
    );
    answer_pull_with(&sb, json!({"objects": [], "removed": [], "sync": "t1"}));
    assert!(sb.output(&["pull"]).status.success());
    answer_pull_with(
        &sb,
        json!({"objects": [task("i:1", "call Bob", None)], "removed": [], "sync": "t2"}),
    );
    assert!(sb.output(&["pull", "--json"]).status.success());
    sb
}

#[test]
fn dam_status_shows_in_plain_text_what_the_pulled_hook_failed_on() {
    let _guard = support::guard("dam_status_shows_in_plain_text_what_the_pulled_hook_failed_on");
    let sb = pulled_by_a_failing_hook();
    let (ok, out, err) = sb.dam(&["status"]);
    assert!(ok, "{err}");
    let config = sb.config_path().display().to_string();
    let line = format!(
        "task_pulled on the last pull from fake: 1 failed, 0 not reached; \
         first error at {config}:3: no bell"
    );
    assert!(out.contains(&line), "{out}");
}

#[test]
fn dam_status_json_names_the_remote_the_counts_and_the_first_error() {
    let _guard = support::guard("dam_status_json_names_the_remote_the_counts_and_the_first_error");
    let sb = pulled_by_a_failing_hook();
    let status = status_json(&sb, &["--json"]);
    assert_matches_schema(&status, "status.schema.json");
    let config = sb.config_path().display().to_string();
    assert_eq!(
        status["notices"],
        json!([{"kind": "task_pulled_failed", "remote": "fake", "failed": 1, "not_reached": 0,
                "error": {"file": config, "line": 3, "message": "no bell"}}])
    );
}
