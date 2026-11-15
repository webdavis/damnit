use crate::support;

use support::sandbox::Sandbox;

fn rows(sb: &Sandbox, check: &str) -> (Option<i32>, Vec<String>) {
    let answer = sb.doctor(&["doctor", check]);
    let rows = answer
        .out
        .lines()
        .skip(3)
        .take_while(|line| !line.is_empty())
        .map(str::to_string)
        .collect();
    (answer.code, rows)
}

#[test]
fn no_remotes_means_no_helper_to_check() {
    let _guard = support::guard("no_remotes_means_no_helper_to_check");
    let sb = Sandbox::new();
    sb.write_config_lua("return {}\n");
    sb.write_secrets_lua("return { schema = 1 }\n");
    let (code, rows) = rows(&sb, "helpers");
    assert_eq!(code, Some(0));
    assert_eq!(rows, ["  · helpers: no remotes configured"]);
}

#[test]
fn a_check_with_no_remote_rows_carries_an_empty_list_of_rows() {
    let _guard = support::guard("a_check_with_no_remote_rows_carries_an_empty_list_of_rows");
    let sb = Sandbox::new();
    sb.write_config_lua("return {}\n");
    sb.write_secrets_lua("return { schema = 1 }\n");
    let (_, document) = sb.doctor_json(&["doctor"]);
    for check in document["checks"].as_array().unwrap() {
        assert_eq!(check["rows"], serde_json::json!([]), "{check}");
    }
}
