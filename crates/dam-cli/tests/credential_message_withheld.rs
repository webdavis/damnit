mod support;

use support::config_files::Remote;
use support::errors::error_document;
use support::sandbox::Sandbox;
use support::tokens::{grant, grant_to};

const RAISED: &str = "LEAK_hunter2";

fn raising_where(sb: &Sandbox) -> String {
    let secrets = std::fs::read_to_string(sb.secrets_path()).unwrap();
    let raising = secrets.replace(
        "api_token = \"tok-123\"",
        &format!("api_token = function() error({RAISED:?}) end"),
    );
    assert_ne!(
        raising, secrets,
        "the sandbox wrote no api_token to replace"
    );
    sb.write_secrets_lua(&raising);
    let line = 1 + raising.lines().position(|l| l.contains(RAISED)).unwrap();
    format!("{}:{line}", sb.secrets_path().display())
}

fn pull_failed_notices(status: &std::process::Output) -> Vec<String> {
    assert!(status.status.success(), "{status:?}");
    let answer: serde_json::Value = serde_json::from_slice(&status.stdout).unwrap();
    answer["notices"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["kind"] == "pull_failed")
        .map(|n| n["why"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn a_token_pulling_sees_where_the_credential_failed_and_not_its_message() {
    let _guard =
        support::guard("a_token_pulling_sees_where_the_credential_failed_and_not_its_message");
    let sb = Sandbox::new();
    let held = grant_to(&sb, "tool", 1, vec![grant(&["remote:pull"])]);
    let at = raising_where(&sb);

    let out = sb.output_holding(&held, &["pull", "fake"]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "{out:?}");
    assert!(stderr.contains(&at), "{stderr}");
    assert!(stderr.contains("dam pull fake"), "{stderr}");
    assert!(!stderr.contains(RAISED), "{stderr}");

    let out = sb.output_holding(&held, &["pull", "fake", "--json"]);
    let message = error_document(&out)["error"]["message"].to_string();
    assert!(message.contains(&at), "{message}");
    assert!(!message.contains(RAISED), "{message}");
}

#[test]
fn the_owner_pulling_sees_the_credential_function_s_message() {
    let _guard = support::guard("the_owner_pulling_sees_the_credential_function_s_message");
    let sb = Sandbox::new();
    let at = raising_where(&sb);

    let out = sb.output(&["pull", "fake"]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "{out:?}");
    assert!(stderr.contains(&at), "{stderr}");
    assert!(stderr.contains(RAISED), "{stderr}");
}

#[test]
fn a_stored_pull_notice_never_holds_the_credential_function_s_message() {
    let _guard =
        support::guard("a_stored_pull_notice_never_holds_the_credential_function_s_message");
    let sb = Sandbox::new();
    sb.add_remote("fake", Remote::fake().stale("1s"));
    let held = grant_to(
        &sb,
        "tool",
        1,
        vec![grant(&["task:read", "remote:pull", "remote:read"])],
    );
    let at = raising_where(&sb);

    let read = sb.output_holding(&held, &["ls", "--json"]);
    assert!(read.status.success(), "{read:?}");
    let seen = pull_failed_notices(&sb.output_holding(&held, &["status", "--json"]));
    assert_eq!(seen.len(), 1, "{seen:?}");

    let read = sb.output(&["ls", "--json"]);
    assert!(read.status.success(), "{read:?}");
    let seen = pull_failed_notices(&sb.output(&["status", "--json"]));
    assert_eq!(seen.len(), 2, "{seen:?}");
    for why in seen {
        assert!(why.contains(&at), "{why}");
        assert!(why.contains("dam pull fake"), "{why}");
        assert!(!why.contains(RAISED), "{why}");
    }
}
