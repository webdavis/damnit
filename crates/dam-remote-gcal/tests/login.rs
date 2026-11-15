use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};

use dam_helper::testing::{self, Reply};
use serde_json::{Value, json};

const CLIENT_SECRET: &str = "MARKER-client-secret";
const REFRESH: &str = "MARKER-refresh-token";

fn answers(base: &str, credentials: bool) -> Vec<Value> {
    let home = tempfile::tempdir().unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_dam-remote-gcal"));
    command
        .args(["gcal", ""])
        .env_clear()
        .env("HOME", home.path())
        .env("DAM_GCAL_BASE_URL", base)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped());
    if credentials {
        command
            .env("DAM_GCAL_CLIENT_ID", "123.apps")
            .env("DAM_GCAL_CLIENT_SECRET", CLIENT_SECRET)
            .env("DAM_GCAL_REFRESH_TOKEN", REFRESH);
    }
    let mut child = command.spawn().unwrap();
    let mut stdin = child.stdin.take().unwrap();
    stdin
        .write_all(b"{\"cmd\":\"capabilities\"}\n{\"cmd\":\"login\"}\n")
        .unwrap();
    drop(stdin);
    let out = child.wait_with_output().unwrap();
    String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn google(reply: Reply) -> testing::Loopback {
    testing::serve(HashMap::from([("POST /token", vec![reply])]))
}

#[test]
fn a_login_is_one_refresh_grant_and_reads_no_calendar() {
    let _guard = testing::guard("a_login_is_one_refresh_grant_and_reads_no_calendar");
    let server = google(Reply::json(
        200,
        json!({"access_token": "ya29.MARKER-access", "expires_in": 3599}),
    ));
    let lines = answers(&server.base, true);
    assert_eq!(lines[0]["answers_login"], true);
    assert_eq!(lines[1], json!({"signed_in": true}));
    let seen = server.seen();
    assert_eq!(seen.len(), 1, "{seen:?}");
    assert_eq!(
        (seen[0].method.as_str(), seen[0].path.as_str()),
        ("POST", "/token")
    );
    assert_eq!(seen[0].form()["grant_type"], "refresh_token");
}

#[test]
fn a_refused_grant_says_what_google_said_and_no_credential() {
    let _guard = testing::guard("a_refused_grant_says_what_google_said_and_no_credential");
    let server = google(Reply::json(
        400,
        json!({"error": "invalid_grant", "error_description": REFRESH}),
    ));
    let lines = answers(&server.base, true);
    let error = lines[1]["error"].as_str().unwrap();
    assert!(error.contains("invalid_grant"), "{error}");
    for secret in [CLIENT_SECRET, REFRESH] {
        assert!(!error.contains(secret), "{error}");
    }
}

#[test]
fn a_login_without_credentials_names_the_variable_and_asks_nothing() {
    let _guard = testing::guard("a_login_without_credentials_names_the_variable_and_asks_nothing");
    let server = google(Reply::json(200, json!({"access_token": "x"})));
    let lines = answers(&server.base, false);
    let error = lines[1]["error"].as_str().unwrap();
    assert!(error.contains("DAM_GCAL_CLIENT_ID"), "{error}");
    assert!(server.seen().is_empty());
}
