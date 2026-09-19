use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn capabilities_are_answered_without_a_token_and_pull_needs_one() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_dam-remote-todoist"))
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
