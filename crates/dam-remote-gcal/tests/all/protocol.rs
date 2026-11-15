use dam_helper::testing;
use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};

use dam_protocol::{Capabilities, PullResponse, Reply as Answer};
use serde::de::DeserializeOwned;
use testing::Reply;

const CLIENT_SECRET: &str = "GOCSPX-SUPERSECRETCLIENT";
const REFRESH: &str = "1//0gSUPERSECRETREFRESH";
const ACCESS: &str = "ya29.SUPERSECRETACCESS";

fn helper(home: &std::path::Path, args: &[&str], base: &str) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_dam-remote-gcal"));
    command
        .args(args)
        .env_clear()
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join("config"))
        .env("XDG_DATA_HOME", home.join("data"))
        .env("XDG_STATE_HOME", home.join("state"))
        .env("DAM_GCAL_BASE_URL", base)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command
}

fn with_credentials(mut command: Command, remote: &str) -> Command {
    command
        .env(format!("DAM_{remote}_CLIENT_ID"), "123.apps")
        .env(format!("DAM_{remote}_CLIENT_SECRET"), CLIENT_SECRET)
        .env(format!("DAM_{remote}_REFRESH_TOKEN"), REFRESH);
    command
}

fn converse(mut command: Command, requests: &str) -> (Vec<String>, String) {
    let mut child = command.spawn().unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(requests.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let responses = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect();
    (responses, String::from_utf8_lossy(&out.stderr).into_owned())
}

fn read<T: DeserializeOwned>(line: &str) -> Answer<T> {
    serde_json::from_str(line).unwrap()
}

fn google() -> testing::Loopback {
    let mut routes = HashMap::new();
    routes.insert(
        "POST /token",
        vec![Reply::json(
            200,
            serde_json::json!({"access_token": ACCESS, "expires_in": 3599}),
        )],
    );
    routes.insert(
        "GET /calendar/v3/calendars/primary/events",
        vec![Reply::json(
            200,
            serde_json::json!({"summary": "me", "timeZone": "UTC", "items": [{
                "id": "a", "summary": "Standup",
                "start": {"dateTime": "2026-09-23T10:00:00Z"}, "end": {"dateTime": "2026-09-23T11:00:00Z"}
            }]}),
        )],
    );
    testing::serve(routes)
}

#[test]
fn capabilities_need_no_credentials_and_a_pull_names_the_missing_variable_only() {
    let _guard = testing::guard(
        "capabilities_need_no_credentials_and_a_pull_names_the_missing_variable_only",
    );
    let home = tempfile::tempdir().unwrap();
    let google = google();
    let (responses, _) = converse(
        helper(home.path(), &["gcal", ""], &google.base),
        "{\"cmd\":\"capabilities\"}\n{\"cmd\":\"pull\",\"since\":null}\n",
    );
    assert!(
        matches!(read::<Capabilities>(&responses[0]), Answer::Answered(c) if c.kinds.is_empty() && c.incremental)
    );
    let Answer::Failed { error } = read::<PullResponse>(&responses[1]) else {
        panic!("{:?}", responses[1])
    };
    assert!(error.contains("DAM_GCAL_CLIENT_ID"), "{error}");
    assert!(google.seen().is_empty());
}

#[test]
fn a_pull_reaches_the_token_endpoint_then_the_calendar_and_answers_dams_document() {
    let _guard = testing::guard(
        "a_pull_reaches_the_token_endpoint_then_the_calendar_and_answers_dams_document",
    );
    let home = tempfile::tempdir().unwrap();
    let google = google();
    let (responses, stderr) = converse(
        with_credentials(helper(home.path(), &["gcal", ""], &google.base), "GCAL"),
        "{\"cmd\":\"pull\",\"since\":null}\n",
    );
    let Answer::Answered(pull) = read::<PullResponse>(&responses[0]) else {
        panic!("{:?}", responses[0])
    };
    assert_eq!(pull.objects[0].remote_id.as_deref(), Some("primary/a"));
    let seen = google.seen();
    assert_eq!(
        (seen[0].path.as_str(), seen[1].path.as_str()),
        ("/token", "/calendar/v3/calendars/primary/events")
    );
    assert_eq!(
        seen[1].authorization.as_deref(),
        Some(format!("Bearer {ACCESS}").as_str())
    );
    for secret in [CLIENT_SECRET, REFRESH, ACCESS] {
        assert!(!stderr.contains(secret));
    }
}

#[test]
fn any_invocation_but_remote_and_address_is_a_usage_error() {
    let _guard = testing::guard("any_invocation_but_remote_and_address_is_a_usage_error");
    let home = tempfile::tempdir().unwrap();
    for args in [&[][..], &["gcal"][..], &["gcal", "", "extra"][..]] {
        let out = helper(home.path(), args, "http://127.0.0.1:1")
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2), "{args:?}");
        assert!(String::from_utf8_lossy(&out.stderr).contains("usage"));
    }
}
