use crate::support;

use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;

fn with_config(config_lua: &str) -> Sandbox {
    let sb = Sandbox::new();
    sb.write_config_lua(config_lua);
    sb.write_secrets_lua("return { schema = 1 }\n");
    sb
}

#[test]
fn a_name_holding_a_dot_is_found_by_its_whole_name() {
    let _guard = support::guard("a_name_holding_a_dot_is_found_by_its_whole_name");
    let sb = with_config(
        "return { filter = { team = { labels = 'p2' }, ['team.a'] = { labels = 'p1' } } }\n",
    );
    let got = |key: &str| sb.dam(&["config", "get", key]).1;
    assert_eq!(got("filter.team.a"), "  filter.team.a.labels  p1\n");
    assert_eq!(got("filter.team.a.labels"), "  filter.team.a.labels  p1\n");
    assert_eq!(got("filter.team.labels"), "  filter.team.labels  p2\n");
}

#[test]
fn a_key_read_two_ways_is_refused_with_both_readings() {
    let _guard = support::guard("a_key_read_two_ways_is_refused_with_both_readings");
    let sb = with_config(
        "return { filter = { team = { labels = 'p2' }, ['team.labels'] = { labels = 'p1' } } }\n",
    );
    let out = sb.output(&["config", "get", "filter.team.labels"]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert!(out.stdout.is_empty());
    assert_eq!(
        String::from_utf8(out.stderr).unwrap(),
        "dam: filter.team.labels names two settings, filter / team / labels and \
filter / team.labels; run dam config get filter to see both\n"
    );
    let (ok, out, err) = sb.dam(&["config", "get", "filter.team.labels.labels"]);
    assert!(ok, "{err}");
    assert_eq!(out, "  filter.team.labels.labels  p1\n");
}

#[test]
fn a_setting_with_no_value_gets_a_row_saying_so() {
    let _guard = support::guard("a_setting_with_no_value_gets_a_row_saying_so");
    let sb = with_config("return { remote = { todoist = { url = 'todoist::' } } }\n");
    let (ok, out, err) = sb.dam(&["config", "get", "remote.todoist.path"]);
    assert!(ok, "{err}");
    assert_eq!(out, "  remote.todoist.path  not set\n");
    let out = sb.output(&["config", "get", "remote.todoist.path", "-o", "json"]);
    assert!(out.status.success(), "{out:?}");
    let answer: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(answer, serde_json::json!({"value": null, "default": true}));
    assert_matches_schema(&answer, "config-get.schema.json");
}

#[test]
fn parts_named_value_and_default_still_match_the_schema() {
    let _guard = support::guard("parts_named_value_and_default_still_match_the_schema");
    let sb =
        with_config("return { remote = { value = { url = 'a::' }, default = { url = 'b::' } } }\n");
    let out = sb.output(&["config", "get", "remote", "-o", "json"]);
    assert!(out.status.success(), "{out:?}");
    let answer: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_matches_schema(&answer, "config-get.schema.json");
}
