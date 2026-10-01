mod support;

use std::io::Write;
use std::process::{Command, Stdio};

fn helper(args: &[&str]) -> Command {
    let home = std::env::temp_dir().join("dam-remote-todoist-protocol-tests");
    let mut command = Command::new(env!("CARGO_BIN_EXE_dam-remote-todoist"));
    command
        .args(args)
        .env("HOME", &home)
        .env("XDG_CONFIG_HOME", home.join("config"))
        .env("XDG_DATA_HOME", home.join("data"))
        .env("XDG_STATE_HOME", home.join("state"));
    command
}

fn pull_error(command: &mut Command) -> String {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    stdin
        .write_all(b"{\"cmd\":\"pull\",\"since\":null}\n")
        .unwrap();
    drop(stdin);
    let out = child.wait_with_output().unwrap();
    let answer: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    answer["error"].as_str().unwrap_or_default().to_string()
}

#[test]
fn capabilities_are_answered_without_a_token_and_pull_needs_one() {
    let _guard = support::guard("capabilities_are_answered_without_a_token_and_pull_needs_one");
    let mut child = helper(&["todoist", ""])
        .env_remove("DAM_TODOIST_API_TOKEN")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    stdin
        .write_all(b"{\"cmd\":\"capabilities\"}\n{\"cmd\":\"pull\",\"since\":null}\n")
        .unwrap();
    drop(stdin);
    let out = child.wait_with_output().unwrap();
    let lines: Vec<&str> = std::str::from_utf8(&out.stdout).unwrap().lines().collect();
    let caps: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(caps["credentials"], serde_json::json!(["api_token"]));
    assert_eq!(caps["kinds"], serde_json::json!(["task"]));
    let pull: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
    assert!(
        pull["error"]
            .as_str()
            .unwrap()
            .contains("DAM_TODOIST_API_TOKEN")
    );
}

#[test]
fn a_malformed_line_answers_an_error_and_the_loop_keeps_going() {
    let _guard = support::guard("a_malformed_line_answers_an_error_and_the_loop_keeps_going");
    let mut child = helper(&["todoist", ""])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    stdin
        .write_all(b"not json\n{\"cmd\":\"capabilities\"}\n")
        .unwrap();
    drop(stdin);
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let lines: Vec<&str> = std::str::from_utf8(&out.stdout).unwrap().lines().collect();
    assert_eq!(lines.len(), 2);
    let error: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert!(error["error"].as_str().is_some());
    let caps: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
    assert_eq!(caps["kinds"], serde_json::json!(["task"]));
}

#[test]
fn a_remote_under_another_name_reads_its_own_token_and_not_todoists() {
    let _guard = support::guard("a_remote_under_another_name_reads_its_own_token_and_not_todoists");
    let missing = pull_error(
        helper(&["work", ""])
            .env("DAM_TODOIST_API_TOKEN", "tok")
            .env_remove("DAM_WORK_API_TOKEN"),
    );
    assert_eq!(
        missing,
        "DAM_WORK_API_TOKEN is not set; declare api_token under [remote.work]"
    );
    let read = pull_error(
        helper(&["work", ""])
            .env_remove("DAM_TODOIST_API_TOKEN")
            .env("DAM_WORK_API_TOKEN", "tok")
            .env("DAM_TODOIST_BASE_URL", "http://127.0.0.1:1"),
    );
    assert!(!read.contains("is not set"), "{read}");
}

#[test]
fn any_invocation_but_remote_and_address_is_a_usage_error() {
    let _guard = support::guard("any_invocation_but_remote_and_address_is_a_usage_error");
    for args in [&[][..], &["todoist"][..], &["todoist", "", "extra"][..]] {
        let out = helper(args).stdin(Stdio::null()).output().unwrap();
        assert_eq!(out.status.code(), Some(2), "{args:?}");
        assert!(String::from_utf8_lossy(&out.stderr).contains("usage"));
    }
}
