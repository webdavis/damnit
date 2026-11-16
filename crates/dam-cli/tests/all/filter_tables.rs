use crate::support;

use support::sandbox::Sandbox;

#[test]
fn a_filter_still_written_as_query_text_is_refused_with_its_table() {
    let _guard = support::guard("a_filter_still_written_as_query_text_is_refused_with_its_table");
    let sb = Sandbox::new();
    sb.write_config_lua("return { filter = { today = { query = 'due:today' } } }\n");
    let out = sb.output(&["list"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert_eq!(
        String::from_utf8(out.stderr).unwrap(),
        "dam: config: filter.today.query in config.lua is query text, which dam no longer reads; \
         write filter.today = { done = \"any\", due = \"today\" } instead; run dam doctor\n"
    );
}

#[test]
fn a_filter_created_with_flags_lists_what_those_flags_list() {
    let _guard = support::guard("a_filter_created_with_flags_lists_what_those_flags_list");
    let sb = Sandbox::new();
    sb.new_object(&["urgent", "--labels", "bug", "-p", "1"]);
    sb.new_object(&["calm", "--labels", "bug", "-p", "4"]);
    sb.new_object(&["other", "-p", "1"]);
    let (ok, out, err) = sb.dam(&[
        "filter",
        "create",
        "hot",
        "--labels",
        "bug",
        "--priority",
        "..2",
    ]);
    assert!(ok, "{err}");
    assert_eq!(out, "created filter hot: --labels bug --priority ..2\n");
    let (ok, out, err) = sb.dam(&["list", "--filter", "hot", "-o", "json"]);
    assert!(ok, "{err}");
    let listed: serde_json::Value = serde_json::from_str(&out).unwrap();
    let subjects: Vec<&str> = listed["objects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["subject"].as_str().unwrap())
        .collect();
    assert_eq!(subjects, ["urgent"]);
}
