use crate::support;

use serde_json::{Value, json};
use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;

fn with_hooks(hooks: &str) -> Sandbox {
    let sb = Sandbox::new();
    sb.write_config_lua(&format!("return {{ hook = {{ {hooks} }} }}\n"));
    sb.write_secrets_lua("return { schema = 1 }\n");
    sb
}

fn listed(sb: &Sandbox) -> Value {
    let out = sb.output(&["config", "list", "--json"]);
    assert!(out.status.success(), "{out:?}");
    let answer = serde_json::from_slice(&out.stdout).unwrap();
    assert_matches_schema(&answer, "config-list.schema.json");
    answer
}

#[test]
fn each_hook_shows_as_a_function_with_what_a_failure_does() {
    let _guard = support::guard("each_hook_shows_as_a_function_with_what_a_failure_does");
    let sb = with_hooks(
        "task_new = function(task) return nil end, \
         task_done = { run = function() end, on_failure = 'refuse' }",
    );
    assert_eq!(
        listed(&sb)["settings"]["hook"],
        json!({
            "task_new": {"run": {"function": true}, "on_failure": {"value": "warn", "default": true}},
            "task_done": {"run": {"function": true}, "on_failure": {"value": "refuse", "default": false}},
        })
    );
    let (ok, out, err) = sb.dam(&["config", "get", "hook"]);
    assert!(ok, "{err}");
    assert_eq!(
        out,
        "  hook.task_new.run          function\n  hook.task_new.on_failure   warn (default)\n  \
         hook.task_done.run         function\n  hook.task_done.on_failure  refuse\n"
    );
}

#[test]
fn config_validate_names_a_wrong_hook() {
    let _guard = support::guard("config_validate_names_a_wrong_hook");
    let sb = with_hooks("task_done = { run = function() end, on_failure = 'ignore' }");
    let (ok, out, _) = sb.dam(&["config", "validate"]);
    assert!(!ok, "{out}");
    assert!(
        out.contains("on_failure must be \"warn\" or \"refuse\" in hook.task_done of config.lua"),
        "{out}"
    );
}
