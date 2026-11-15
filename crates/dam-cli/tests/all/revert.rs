use crate::support;

use serde_json::{Value, json};
use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to, refused};

fn run(sb: &Sandbox, args: &[&str]) {
    let (ok, _, err) = sb.dam(args);
    assert!(ok, "{args:?}: {err}");
}

fn commit(sb: &Sandbox, message: &str) -> String {
    run(sb, &["add", "-A"]);
    run(sb, &["commit", "-m", message]);
    let (_, log, _) = sb.dam(&["log", "--json"]);
    let log: Value = serde_json::from_str(&log).unwrap();
    log["commits"][0]["id"].as_str().unwrap().to_string()
}

fn revert(sb: &Sandbox, args: &[&str]) -> Value {
    let line = [&["revert"], args, &["--json"]].concat();
    answer(&sb.output(&line), "revert.schema.json")
}

fn subject(sb: &Sandbox, oid: &str) -> Option<String> {
    let out = sb.output(&["show", oid, "--json"]);
    let shown: Value = serde_json::from_slice(&out.stdout).ok()?;
    shown["subject"].as_str().map(str::to_string)
}

fn row(oid: &str, fields: &[&str]) -> Value {
    json!({ "oid": oid, "fields": fields })
}

#[test]
fn a_field_still_as_committed_goes_back_and_one_changed_since_is_kept() {
    let _guard =
        support::guard("a_field_still_as_committed_goes_back_and_one_changed_since_is_kept");
    let sb = Sandbox::new();
    let oid = sb.new_object(&["call Ann"]);
    commit(&sb, "add");
    run(
        &sb,
        &["edit", &oid, "--subject", "call Bob", "--priority", "1"],
    );
    let edited = commit(&sb, "edit");
    run(&sb, &["edit", &oid, "--subject", "call Ada"]);

    let out = revert(&sb, &[&edited]);

    let full_oid = out["reverted"][0]["oid"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    assert!(full_oid.starts_with(&oid), "{out}");
    assert_eq!(out["reverted"], json!([row(&full_oid, &["priority"])]));
    assert_eq!(out["kept"], json!([row(&full_oid, &["subject"])]));
    assert_eq!(subject(&sb, &oid).as_deref(), Some("call Ada"));
    let (ok, human, err) = sb.dam(&["diff"]);
    assert!(ok && human.contains("priority"), "{human}{err}");
}

#[test]
fn naming_one_task_of_two_reverts_only_that_one() {
    let _guard = support::guard("naming_one_task_of_two_reverts_only_that_one");
    let sb = Sandbox::new();
    let ann = sb.new_object(&["call Ann"]);
    let bob = sb.new_object(&["call Bob"]);
    commit(&sb, "add");
    run(&sb, &["edit", &ann, "--subject", "call Ann at 5"]);
    run(&sb, &["edit", &bob, "--subject", "call Bob at 5"]);
    let edited = commit(&sb, "edit");

    let out = revert(&sb, &[&edited, &bob]);

    assert_eq!(out["reverted"].as_array().map(Vec::len), Some(1), "{out}");
    assert_eq!(subject(&sb, &ann).as_deref(), Some("call Ann at 5"));
    assert_eq!(subject(&sb, &bob).as_deref(), Some("call Bob"));
}

#[test]
fn a_created_task_untouched_since_is_removed_and_one_edited_since_is_kept() {
    let _guard =
        support::guard("a_created_task_untouched_since_is_removed_and_one_edited_since_is_kept");
    let sb = Sandbox::new();
    let ann = sb.new_object(&["call Ann"]);
    let bob = sb.new_object(&["call Bob"]);
    let created = commit(&sb, "add");
    run(&sb, &["edit", &bob, "--subject", "call Bob at 5"]);

    let out = revert(&sb, &[&created]);

    assert_eq!(out["reverted"].as_array().map(Vec::len), Some(1), "{out}");
    assert_eq!(out["kept"].as_array().map(Vec::len), Some(1), "{out}");
    assert_eq!(subject(&sb, &ann), None);
    assert_eq!(subject(&sb, &bob).as_deref(), Some("call Bob at 5"));
}

#[test]
fn a_token_is_refused() {
    let _guard = support::guard("a_token_is_refused");
    let sb = Sandbox::new();
    sb.new_object(&["call Ann"]);
    let added = commit(&sb, "add");
    let held = grant_to(&sb, "tool", 1, vec![grant(&["task:read", "history:read"])]);

    let out = sb.output_holding(&held, &["revert", &added, "--json"]);

    assert_eq!(refused(&out), "not_open_to_tokens");
    assert_eq!(
        support::status_json(&sb, &["--json"])["unstaged"],
        json!([])
    );
}
