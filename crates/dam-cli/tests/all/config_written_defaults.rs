use crate::support;

use serde_json::json;
use support::sandbox::Sandbox;

fn written(config_lua: &str) -> Sandbox {
    let sb = Sandbox::new();
    sb.write_config_lua(config_lua);
    sb.write_secrets_lua("return { schema = 1 }\n");
    sb
}

fn listed(sb: &Sandbox) -> (String, serde_json::Value) {
    let (ok, text, err) = sb.dam(&["config", "list"]);
    assert!(ok, "{err}");
    let out = sb.output(&["config", "list", "-o", "json"]);
    assert!(out.status.success(), "{out:?}");
    (text, serde_json::from_slice(&out.stdout).unwrap())
}

#[test]
fn a_false_the_file_writes_is_not_a_default() {
    let _guard = support::guard("a_false_the_file_writes_is_not_a_default");
    let sb = written(
        "return {\n  done = { interactive = false },\n  category = { effort = { values = { 'deep' }, exclusive = false } },\n}\n",
    );
    let (text, answer) = listed(&sb);
    assert!(
        text.contains("  done.interactive           false\n"),
        "{text}"
    );
    assert!(
        text.contains("  category.effort.exclusive  false\n"),
        "{text}"
    );
    let settings = &answer["settings"];
    assert_eq!(
        settings["done"]["interactive"],
        json!({"value": false, "default": false})
    );
    assert_eq!(
        settings["category"]["effort"]["exclusive"],
        json!({"value": false, "default": false})
    );
}

#[test]
fn a_false_the_file_leaves_out_is_dams_default() {
    let _guard = support::guard("a_false_the_file_leaves_out_is_dams_default");
    let sb = written("return {\n  category = { effort = { values = { 'deep' } } },\n}\n");
    let (text, answer) = listed(&sb);
    assert!(
        text.contains("  done.interactive           false (default)\n"),
        "{text}"
    );
    assert!(
        text.contains("  category.effort.exclusive  false (default)\n"),
        "{text}"
    );
    let settings = &answer["settings"];
    assert_eq!(
        settings["done"]["interactive"],
        json!({"value": false, "default": true})
    );
    assert_eq!(
        settings["category"]["effort"]["exclusive"],
        json!({"value": false, "default": true})
    );
}
