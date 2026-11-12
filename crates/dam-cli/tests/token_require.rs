mod support;

use support::errors::error_document;
use support::sandbox::Sandbox;
use support::tokens::grant_to;

fn requiring_session_and_model(sb: &Sandbox) -> String {
    grant_to(
        sb,
        "claude",
        1,
        "require = [\"session\", \"model\"]\n[[token.claude.grant]]\nallow = [\"task:read\"]\n",
    )
}

#[test]
fn a_request_missing_a_required_detail_is_refused_naming_only_what_to_pass() {
    let _guard =
        support::guard("a_request_missing_a_required_detail_is_refused_naming_only_what_to_pass");
    let sb = Sandbox::new();
    let held = requiring_session_and_model(&sb);
    let out = sb.output_holding(
        &held,
        &["ls", "--json", "--no-pull", "--meta", "session=s1"],
    );
    assert_eq!(out.status.code(), Some(5), "{out:?}");
    let document = error_document(&out);
    assert_eq!(document["error"]["rule"], "missing_details");
    let message = document["error"]["message"].as_str().unwrap();
    assert!(message.contains("--meta model="), "{message}");
    assert!(!message.contains("session"), "{message}");
}

#[test]
fn a_request_carrying_every_required_detail_is_served() {
    let _guard = support::guard("a_request_carrying_every_required_detail_is_served");
    let sb = Sandbox::new();
    let held = requiring_session_and_model(&sb);
    let out = sb.output_holding(
        &held,
        &[
            "ls",
            "--json",
            "--no-pull",
            "--meta",
            "session=s1",
            "--meta",
            "model=opus",
        ],
    );
    assert!(out.status.success(), "{out:?}");
}

#[test]
fn a_meta_without_an_equals_sign_is_a_usage_error() {
    let _guard = support::guard("a_meta_without_an_equals_sign_is_a_usage_error");
    let sb = Sandbox::new();
    let out = sb.output(&["ls", "--no-pull", "--meta", "session"]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
}

#[test]
fn token_create_writes_what_the_token_requires() {
    let _guard = support::guard("token_create_writes_what_the_token_requires");
    let sb = Sandbox::new();
    let (ok, _, err) = sb.dam(&[
        "token",
        "create",
        "claude",
        "--allow",
        "task:read",
        "--require",
        "session",
        "--require",
        "model",
    ]);
    assert!(ok, "{err}");
    let (ok, out, err) = sb.dam(&["token", "list", "--json"]);
    assert!(ok, "{err}");
    let listed: serde_json::Value = serde_json::from_str(&out).unwrap();
    let claude = &listed["tokens"][0];
    assert_eq!(claude["require"], serde_json::json!(["session", "model"]));
    assert_eq!(claude["standing"], "active");
}
