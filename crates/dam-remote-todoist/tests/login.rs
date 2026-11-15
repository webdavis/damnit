use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};

use dam_helper::testing::{self, Reply};
use serde_json::{Value, json};

const MARKER: &str = "MARKER-7f3a-api-token";

fn answers(base: &str, token: Option<&str>) -> Vec<Value> {
    let home = tempfile::tempdir().unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_dam-remote-todoist"));
    command
        .args(["todoist", ""])
        .env_clear()
        .env("HOME", home.path())
        .env("DAM_TODOIST_BASE_URL", base)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped());
    if let Some(token) = token {
        command.env("DAM_TODOIST_API_TOKEN", token);
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

fn todoist(reply: Reply) -> testing::Loopback {
    testing::serve(HashMap::from([("GET /projects", vec![reply])]))
}

#[test]
fn a_login_reads_one_project_with_the_token_and_writes_nothing() {
    let _guard = testing::guard("a_login_reads_one_project_with_the_token_and_writes_nothing");
    let server = todoist(Reply::json(
        200,
        json!({"results": [{"id": "p1", "name": "Inbox"}], "next_cursor": null}),
    ));
    let lines = answers(&server.base, Some(MARKER));
    assert_eq!(lines[0]["answers_login"], true);
    assert_eq!(lines[1], json!({"signed_in": true}));
    let seen = server.seen();
    assert_eq!(seen.len(), 1, "{seen:?}");
    assert_eq!(
        (seen[0].method.as_str(), seen[0].path.as_str()),
        ("GET", "/projects")
    );
    assert_eq!(seen[0].query, "limit=1");
    assert!(seen[0].body.is_empty());
    let bearer = format!("Bearer {MARKER}");
    assert_eq!(seen[0].authorization.as_deref(), Some(bearer.as_str()));
}

#[test]
fn a_refused_token_answers_what_todoist_said_without_the_token() {
    let _guard = testing::guard("a_refused_token_answers_what_todoist_said_without_the_token");
    let server = todoist(Reply {
        status: 401,
        body: format!("Unauthorized: token {MARKER} is not valid"),
        headers: vec![],
    });
    let lines = answers(&server.base, Some(MARKER));
    let error = lines[1]["error"].as_str().unwrap();
    assert!(error.contains("401"), "{error}");
    assert!(error.contains("Unauthorized: token"), "{error}");
    assert!(error.contains("is not valid"), "{error}");
    assert!(!error.contains(MARKER), "{error}");
}

#[test]
fn a_login_without_a_token_names_the_variable_and_asks_nothing() {
    let _guard = testing::guard("a_login_without_a_token_names_the_variable_and_asks_nothing");
    let server = todoist(Reply::json(200, json!({"results": []})));
    let lines = answers(&server.base, None);
    let error = lines[1]["error"].as_str().unwrap();
    assert!(error.contains("DAM_TODOIST_API_TOKEN"), "{error}");
    assert!(server.seen().is_empty());
}
