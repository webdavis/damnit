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
fn the_pulled_hook_shows_as_a_function() {
    let _guard = support::guard("the_pulled_hook_shows_as_a_function");
    let sb = with_hooks("task_pulled = { run = function(task, before, pull) end }");
    assert_eq!(
        listed(&sb)["settings"]["hook"],
        json!({"task_pulled": {
            "run": {"function": true},
            "changes": {"value": "committed", "default": true},
        }})
    );
    let (ok, out, err) = sb.dam(&["config", "get", "hook"]);
    assert!(ok, "{err}");
    assert_eq!(
        out,
        "  hook.task_pulled.run      function\n  hook.task_pulled.changes  committed (default)\n"
    );
}

#[test]
fn config_validate_says_the_pulled_hook_always_warns() {
    let _guard = support::guard("config_validate_says_the_pulled_hook_always_warns");
    let sb = with_hooks("task_pulled = { run = function() end, on_failure = 'refuse' }");
    let (ok, out, _) = sb.dam(&["config", "validate"]);
    assert!(!ok, "{out}");
    assert!(
        out.contains(
            "on_failure is not taken in hook.task_pulled of config.lua: that hook always warns"
        ),
        "{out}"
    );
}
