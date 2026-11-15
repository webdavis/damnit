mod support;

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
