use crate::support;

use serde_json::{Value, json};
use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;

fn with_filters(filters: &str) -> Sandbox {
    let sb = Sandbox::new();
    sb.write_config_lua(&format!("return {{ filter = {{ {filters} }} }}\n"));
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
fn a_filter_function_shows_as_a_function_and_never_its_source() {
    let _guard = support::guard("a_filter_function_shows_as_a_function_and_never_its_source");
    let sb = with_filters(
        "work = { match = function(task) return task.path == 'work' end }, \
         soon = { query = 'due:today', match = function() return true end }",
    );
    assert_eq!(
        listed(&sb)["settings"]["filter"],
        json!({
            "soon": {"query": {"value": "due:today", "default": false}, "match": {"function": true}},
            "work": {"match": {"function": true}},
        })
    );
    let (ok, out, err) = sb.dam(&["config", "get", "filter"]);
    assert!(ok, "{err}");
    assert_eq!(
        out,
        "  filter.soon.query  due:today\n  filter.soon.match  function\n  filter.work.match  function\n"
    );
}

#[test]
fn a_filter_named_function_is_still_a_part() {
    let _guard = support::guard("a_filter_named_function_is_still_a_part");
    let sb = with_filters("['function'] = { match = function() return true end }");
    assert_eq!(
        listed(&sb)["settings"]["filter"]["function"],
        json!({"match": {"function": true}})
    );
}
