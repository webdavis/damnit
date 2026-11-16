use crate::support;

use serde_json::{Value, json};
use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;

fn with_remotes(remotes: &str) -> Sandbox {
    let sb = Sandbox::new();
    sb.write_config_lua(&format!("return {{ remote = {{ {remotes} }} }}\n"));
    sb.write_secrets_lua("return { schema = 1 }\n");
    sb
}

fn listed(sb: &Sandbox) -> Value {
    let out = sb.output(&["config", "list", "-o", "json"]);
    assert!(out.status.success(), "{out:?}");
    serde_json::from_slice(&out.stdout).unwrap()
}

#[test]
fn a_setting_with_no_value_has_a_row_saying_so() {
    let _guard = support::guard("a_setting_with_no_value_has_a_row_saying_so");
    let sb = with_remotes("todoist = { url = 'todoist::' }");
    let (ok, out, err) = sb.dam(&["config", "list"]);
    assert!(ok, "{err}");
    let settings = out.split_once("Settings ──").unwrap().1;
    assert!(
        settings.ends_with(
            "\
  remote.todoist.url            todoist::
  remote.todoist.deadline       60s (default)
  remote.todoist.sync.interval  15m (default)
  remote.todoist.sync.allowed   true (default)
  remote.todoist.path           not set
  remote.todoist.except         not set
"
        ),
        "{out}"
    );
    let answer = listed(&sb);
    assert_matches_schema(&answer, "config-list.schema.json");
    let todoist = &answer["settings"]["remote"]["todoist"];
    assert_eq!(todoist["path"], json!({"value": null, "default": true}));
    assert_eq!(todoist["except"], json!({"value": null, "default": true}));
}

#[test]
fn parts_named_value_and_default_still_match_the_schema() {
    let _guard = support::guard("parts_named_value_and_default_still_match_the_schema");
    let sb = with_remotes("value = { url = 'a::' }, default = { url = 'b::' }");
    assert_matches_schema(&listed(&sb), "config-list.schema.json");
}
