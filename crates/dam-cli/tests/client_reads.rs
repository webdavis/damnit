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

/// A helper is any program that speaks the protocol, so a completion time on
/// an open task has to be refused where the object is read rather than where
/// a declared field is merged.
#[test]
fn a_helper_cannot_put_a_completion_time_on_an_open_task() {
    let _guard = support::guard("a_helper_cannot_put_a_completion_time_on_an_open_task");
    let sb = Sandbox::new();
    let task = r#"{"oid":"","remote_id":"up-1","kind":"task","subject":"upstream","body":"","path":"","labels":[],"depends":[],"reminders":[],"recurrence":null,"task":{"done":false,"completed_at":"2020-01-01T00:00:00Z","priority":2,"due":null,"deadline":null,"event":null},"event":null}"#;
    let helper = sb.dir.path().join("bin/dam-remote-fake");
    std::fs::write(
        &helper,
        format!(
            r#"#!/bin/sh
set -eu
while IFS= read -r line; do
  case "$line" in
    *'"capabilities"'*) printf '{{"protocol":1,"kinds":["task"],"fields":["subject"],"credentials":["api_token"],"incremental":false}}\n' ;;
    *'"pull"'*) printf '{{"objects":[{task}],"removed":[],"sync":null}}\n' ;;
    *) printf '{{"error":"unknown request"}}\n' ;;
  esac
done
"#
        ),
    )
    .unwrap();
    std::fs::set_permissions(&helper, std::os::unix::fs::PermissionsExt::from_mode(0o755)).unwrap();

    let (ok, _, err) = sb.dam(&["pull", "--json"]);
    assert!(ok, "{err}");
    let (ok, out, err) = sb.dam(&["ls", "--json", "--no-pull"]);
    assert!(ok, "{err}");
    let answer: serde_json::Value = serde_json::from_str(&out).unwrap();
    let task = &answer["objects"][0]["task"];
    assert_eq!(task["done"], false, "{answer}");
    assert_eq!(
        task["completed_at"],
        serde_json::Value::Null,
        "an open task was stored carrying a completion time: {answer}"
    );
}

/// `--no-pull` says which reads answer locally, so a verb that never reaches a
/// remote refuses it rather than accepting a flag that means nothing there.
#[test]
fn no_pull_is_refused_on_a_verb_that_never_pulls() {
    let _guard = support::guard("no_pull_is_refused_on_a_verb_that_never_pulls");
    let sb = Sandbox::new();
    let oid = sb.new_object(&["buy oat milk"]);

    for args in [
        vec!["pull", "--no-pull"],
        vec!["status", "--no-pull"],
        vec!["done", oid.as_str(), "--no-pull"],
        vec!["category", "list", "--no-pull"],
        vec!["filter", "list", "--no-pull"],
    ] {
        let out = sb.output(&args);
        assert_eq!(
            out.status.code(),
            Some(2),
            "{args:?} took a flag it cannot act on"
        );
        let said = String::from_utf8_lossy(&out.stderr);
        assert!(said.contains("--no-pull"), "{said}");
    }

    for args in [
        vec!["ls", "--no-pull"],
        vec!["show", oid.as_str(), "--no-pull"],
    ] {
        let out = sb.output(&args);
        assert_eq!(out.status.code(), Some(0), "{args:?} refused the flag");
    }
}

/// The catalogue a client renders: two listing verbs, so no client owns a
/// second parser for the config file `dam` owns.
#[test]
fn category_list_and_filter_list_print_what_config_declares() {
    let _guard = support::guard("category_list_and_filter_list_print_what_config_declares");
    let sb = Sandbox::new();
    let config = sb.dir.path().join("config.toml");
    let mut text = std::fs::read_to_string(&config).unwrap();
    text.push_str(
        "\n[category.effort]\nvalues = [\"light\", \"admin\", \"deep\"]\nexclusive = true\n\
         \n[category.context]\nvalues = [\"home\", \"errand\"]\n\
         \n[filter.today]\nquery = \"due:today | overdue\"\n",
    );
    std::fs::write(&config, text).unwrap();

    let (ok, out, err) = sb.dam(&["category", "list", "--json"]);
    assert!(ok, "{err}");
    let answer: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        answer,
        serde_json::json!({"categories": [
            {"name": "context", "values": ["home", "errand"], "exclusive": false},
            {"name": "effort", "values": ["light", "admin", "deep"], "exclusive": true},
        ]}),
        "{answer}"
    );

    let (ok, out, err) = sb.dam(&["filter", "list", "--json"]);
    assert!(ok, "{err}");
    let answer: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        answer,
        serde_json::json!({"filters": [{"name": "today", "query": "due:today | overdue"}]}),
        "{answer}"
    );

    let (ok, out, err) = sb.dam(&["category", "list"]);
    assert!(ok, "{err}");
    assert!(out.contains("effort"), "{out}");
    assert!(out.contains("light, admin, deep"), "{out}");
    let (ok, out, err) = sb.dam(&["filter", "list"]);
    assert!(ok, "{err}");
    assert!(out.contains("due:today | overdue"), "{out}");
}

/// A store with nothing declared answers with an empty list rather than a
/// failure, so a client renders an empty picker instead of an error.
#[test]
fn an_empty_catalogue_is_an_empty_array() {
    let _guard = support::guard("an_empty_catalogue_is_an_empty_array");
    let sb = Sandbox::new();

    for (args, key) in [
        (["category", "list", "--json"], "categories"),
        (["filter", "list", "--json"], "filters"),
    ] {
        let (ok, out, err) = sb.dam(&args);
        assert!(ok, "{err}");
        let answer: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(answer[key], serde_json::json!([]), "{answer}");
    }
}

/// TOON is the same answer in the compact form, which is a table per list.
#[test]
fn the_catalogue_renders_as_toon_too() {
    let _guard = support::guard("the_catalogue_renders_as_toon_too");
    let sb = Sandbox::new();
    let config = sb.dir.path().join("config.toml");
    let mut text = std::fs::read_to_string(&config).unwrap();
    text.push_str("\n[filter.today]\nquery = \"due:today\"\n");
    std::fs::write(&config, text).unwrap();

    let (ok, out, err) = sb.dam(&["filter", "list", "--toon"]);
    assert!(ok, "{err}");
    assert!(out.starts_with("filters[1]{name,query}:"), "{out}");
}
