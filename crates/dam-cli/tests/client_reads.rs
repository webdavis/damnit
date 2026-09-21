//! The reads a client makes per render: answering locally, reporting how
//! fresh each remote is, and the completion time a Done list shows.

mod support;

use support::sandbox::Sandbox;
use support::status_json;

/// `--no-pull` is what lets a client promise a render costs no network: the
/// same read pulls the stale remote without it and answers locally with it.
#[test]
fn no_pull_answers_a_read_from_the_store_alone() {
    let _guard = support::guard("no_pull_answers_a_read_from_the_store_alone");
    let sb = Sandbox::new();
    let config = sb.dir.path().join("config.toml");
    let mut text = std::fs::read_to_string(&config).unwrap();
    text.push_str("stale = \"15m\"\n");
    std::fs::write(&config, text).unwrap();

    let (ok, out, err) = sb.dam(&["ls", "--json", "--no-pull"]);
    assert!(ok, "{err}");
    let answer: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        answer["objects"].as_array().unwrap().len(),
        0,
        "a local read reached the remote: {answer}"
    );

    let (ok, out, err) = sb.dam(&["ls", "--json"]);
    assert!(ok, "{err}");
    let answer: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        answer["objects"].as_array().unwrap().len(),
        1,
        "the same read without the flag pulls the stale remote: {answer}"
    );
}

/// A client header reads freshness off `remote list`: RFC 3339 per remote in
/// the document, null until that verb has reached the remote once.
#[test]
fn remote_list_reports_when_each_remote_was_last_pulled_and_pushed() {
    let _guard = support::guard("remote_list_reports_when_each_remote_was_last_pulled_and_pushed");
    let sb = Sandbox::new();

    let fresh = remote_row(&sb);
    assert_eq!(fresh["last_pull"], serde_json::Value::Null, "{fresh}");
    assert_eq!(fresh["last_push"], serde_json::Value::Null, "{fresh}");

    let (ok, _, err) = sb.dam(&["pull", "--json"]);
    assert!(ok, "{err}");
    let pulled = remote_row(&sb);
    assert!(pulled["last_pull"].as_str().is_some(), "{pulled}");
    assert_eq!(pulled["last_push"], serde_json::Value::Null, "{pulled}");

    sb.new_object(&["buy oat milk"]);
    let (ok, _, err) = sb.dam(&["add", "-A"]);
    assert!(ok, "{err}");
    let (ok, _, err) = sb.dam(&["commit", "-m", "triage"]);
    assert!(ok, "{err}");
    let (ok, _, err) = sb.dam(&["push", "--json"]);
    assert!(ok, "{err}");
    let pushed = remote_row(&sb);
    let at = pushed["last_push"].as_str().expect("a push time");
    at.parse::<jiff::Timestamp>()
        .unwrap_or_else(|e| panic!("last_push is not RFC 3339: {e} ({at})"));
    assert_eq!(pushed["last_pull"], pulled["last_pull"], "{pushed}");

    let (ok, human, err) = sb.dam(&["remote", "list"]);
    assert!(ok, "{err}");
    assert!(human.contains("pulled "), "{human}");
    assert!(human.contains("pushed "), "{human}");
}

/// The one remote a sandbox configures, as `remote list --json` reports it.
fn remote_row(sb: &Sandbox) -> serde_json::Value {
    let (ok, out, err) = sb.dam(&["remote", "list", "--json"]);
    assert!(ok, "{err}");
    let answer: serde_json::Value = serde_json::from_str(&out).unwrap();
    answer["remotes"][0].clone()
}

/// `dam done` records when, `dam edit --undone` takes it away, and both the
/// object document and the change row carry it.
#[test]
fn a_completion_time_appears_in_the_object_and_the_change_and_is_cleared_by_undone() {
    let _guard = support::guard(
        "a_completion_time_appears_in_the_object_and_the_change_and_is_cleared_by_undone",
    );
    let sb = Sandbox::new();
    let oid = sb.new_object(&["buy oat milk"]);

    let (ok, out, err) = sb.dam(&["show", &oid, "--json"]);
    assert!(ok, "{err}");
    let open: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        open["task"]["completed_at"],
        serde_json::Value::Null,
        "{open}"
    );

    let (ok, _, err) = sb.dam(&["done", &oid]);
    assert!(ok, "{err}");
    let (ok, out, err) = sb.dam(&["show", &oid, "--json"]);
    assert!(ok, "{err}");
    let done: serde_json::Value = serde_json::from_str(&out).unwrap();
    let at = done["task"]["completed_at"]
        .as_str()
        .unwrap_or_else(|| panic!("no completion time: {done}"));
    at.parse::<jiff::Timestamp>()
        .unwrap_or_else(|e| panic!("completed_at is not RFC 3339: {e} ({at})"));

    let row = status_json(&sb, &["--json"])["unstaged"][0].clone();
    assert_eq!(row["done"], true, "{row}");
    assert_eq!(row["completed_at"], at, "{row}");

    let (ok, _, err) = sb.dam(&["edit", &oid, "--undone"]);
    assert!(ok, "{err}");
    let (ok, out, err) = sb.dam(&["show", &oid, "--json"]);
    assert!(ok, "{err}");
    let reopened: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        reopened["task"]["completed_at"],
        serde_json::Value::Null,
        "{reopened}"
    );
}

/// A client that completes a task twice, because the operator clicked twice or
/// a retry repeated the call, leaves the workspace clean.
#[test]
fn completing_a_task_twice_leaves_nothing_to_report() {
    let _guard = support::guard("completing_a_task_twice_leaves_nothing_to_report");
    let sb = Sandbox::new();
    let oid = sb.new_object(&["repeat done test"]);
    assert!(sb.dam(&["done", &oid]).0);
    assert!(sb.dam(&["add", "-A"]).0);
    assert!(sb.dam(&["commit", "-m", "done it"]).0);

    assert!(sb.dam(&["done", &oid]).0);
    assert_eq!(
        status_json(&sb, &["--json"])["unstaged"],
        serde_json::json!([]),
        "the second done made a change describing nothing"
    );
}
