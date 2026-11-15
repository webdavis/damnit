use crate::support;

use std::os::unix::fs::PermissionsExt;

use support::config_files::Remote;
use support::sandbox::Sandbox;

const A_DEADLINE_A_BUSY_MACHINE_MEETS: &str = "the helper is silent until its one second deadline";

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

fn install_helper(sb: &Sandbox, script: &str) {
    let helper = sb.dir.path().join("bin/dam-remote-fake");
    std::fs::write(&helper, script).unwrap();
    std::fs::set_permissions(&helper, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn a_helper_that_answers_is_well_and_is_given_no_credential() {
    let _guard = support::guard("a_helper_that_answers_is_well_and_is_given_no_credential");
    let sb = Sandbox::new();
    let (code, rows) = rows(&sb, "helpers");
    assert_eq!(code, Some(0));
    assert_eq!(
        rows,
        [
            "  ✓ helpers: 1 remote: 1 ok",
            "    ✓ fake: dam-remote-fake answered",
        ]
    );
    let given = std::fs::read_to_string(sb.dir.path().join("token.txt")).unwrap();
    assert_eq!(given, "\n");
    assert!(sb.pushes().is_empty());
    assert!(sb.no_helper_left_within(std::time::Duration::from_secs(2)));
}

#[test]
fn a_missing_helper_is_an_issue_with_advice() {
    let _guard = support::guard("a_missing_helper_is_an_issue_with_advice");
    let sb = Sandbox::new();
    std::fs::remove_file(sb.dir.path().join("bin/dam-remote-fake")).unwrap();
    let (code, rows) = rows(&sb, "helpers");
    assert_eq!(code, Some(1));
    assert_eq!(
        rows,
        [
            "  ✗ helpers: 1 remote: 1 issue",
            "    ✗ fake: dam-remote-fake is not installed",
            "      → install it, then run dam doctor",
        ]
    );
}

#[test]
fn a_helper_on_another_protocol_is_an_issue() {
    let _guard = support::guard("a_helper_on_another_protocol_is_an_issue");
    let sb = Sandbox::new();
    install_helper(
        &sb,
        "#!/bin/sh\nread -r line\nprintf '{\"protocol\":99,\"kinds\":[\"task\"],\"fields\":[],\"credentials\":[],\"incremental\":false}\\n'\n",
    );
    let (code, rows) = rows(&sb, "helpers");
    assert_eq!(code, Some(1));
    assert!(
        rows[1].starts_with("    ✗ fake: helper fake speaks protocol version 99"),
        "{rows:?}"
    );
}

#[test]
fn a_silent_helper_is_an_issue_once_its_deadline_passes() {
    let _guard = support::guard("a_silent_helper_is_an_issue_once_its_deadline_passes")
        .allow_slow(A_DEADLINE_A_BUSY_MACHINE_MEETS);
    let sb = Sandbox::new();
    sb.add_remote("fake", Remote::fake().deadline("1s"));
    sb.install_silent_helper();
    let (code, rows) = rows(&sb, "helpers");
    assert_eq!(code, Some(1));
    assert!(
        rows[1].starts_with("    ✗ fake: helper fake gave no answer within 1s"),
        "{rows:?}"
    );
    assert!(sb.no_helper_left_within(std::time::Duration::from_secs(2)));
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
fn helpers_are_not_checked_when_the_config_does_not_read() {
    let _guard = support::guard("helpers_are_not_checked_when_the_config_does_not_read");
    let sb = Sandbox::new();
    sb.write_config_lua("return {\n");
    let (code, rows) = rows(&sb, "helpers");
    assert_eq!(code, Some(0));
    assert_eq!(rows, ["  · helpers: not checked, the config did not read"]);
    assert!(!sb.dir.path().join("token.txt").exists());
}

#[test]
fn each_remote_is_a_row_named_by_the_remote_under_its_check() {
    let _guard = support::guard("each_remote_is_a_row_named_by_the_remote_under_its_check");
    let sb = Sandbox::new();
    sb.add_remote("work", Remote::at("nosuch::"));
    let (code, rows) = rows(&sb, "helpers");
    assert_eq!(code, Some(1));
    assert_eq!(
        rows,
        [
            "  ✗ helpers: 2 remotes: 1 issue, 1 ok",
            "    ✓ fake: dam-remote-fake answered",
            "    ✗ work: dam-remote-nosuch is not installed",
            "      → install it, then run dam doctor",
        ]
    );
    let (_, document) = sb.doctor_json(&["doctor", "helpers"]);
    assert_eq!(
        document["checks"],
        serde_json::json!([{
            "name": "helpers",
            "result": "issue",
            "detail": "2 remotes: 1 issue, 1 ok",
            "advice": null,
            "rows": [
                {"name": "fake", "result": "ok", "detail": "dam-remote-fake answered", "advice": null},
                {
                    "name": "work",
                    "result": "issue",
                    "detail": "dam-remote-nosuch is not installed",
                    "advice": "install it, then run dam doctor"
                }
            ]
        }])
    );
    assert_eq!(
        document["needs_attention"],
        serde_json::json!(["helpers for work: dam-remote-nosuch is not installed"])
    );
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
