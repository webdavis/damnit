use crate::support;

use serde_json::{Value, json};
use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;
use support::upstream::{answer_pull_with, task};

const FILING: &str = "function(task, before, pull)\n  \
    print('pulled ' .. task.subject .. ' ' .. pull.change .. ' from ' .. pull.remote)\n  \
    return { subject = task.subject .. ' today' }\n\
  end";

fn hooked(hook: &str) -> Sandbox {
    let sb = Sandbox::new();
    sb.write_config_lua(&format!(
        "return {{\n  remote = {{ fake = {{ url = 'fake::' }} }},\n  \
         hook = {{ task_pulled = {hook} }},\n}}\n"
    ));
    answer_pull_with(
        &sb,
        json!({"objects": [task("i:1", "water the plants", None)], "removed": [], "sync": "t1"}),
    );
    let first = pulled(&sb);
    assert_eq!(first["remotes"][0]["created"], 1, "{first}");
    assert!(first["remotes"][0].get("task_pulled").is_none(), "{first}");
    answer_pull_with(
        &sb,
        json!({"objects": [task("i:2", "call Bob", None)], "removed": [], "sync": "t2"}),
    );
    sb
}

fn pulled(sb: &Sandbox) -> Value {
    let out = sb.output(&["pull", "--json"]);
    assert!(out.status.success(), "{out:?}");
    let answer = serde_json::from_slice(&out.stdout).unwrap();
    assert_matches_schema(&answer, "pull.schema.json");
    answer
}

fn subjects(sb: &Sandbox) -> Vec<String> {
    let (ok, out, err) = sb.dam(&["ls", "--json"]);
    assert!(ok, "{err}");
    let listed: Value = serde_json::from_str(&out).unwrap();
    let objects = listed["objects"].as_array().unwrap().iter();
    objects
        .map(|o| o["subject"].as_str().unwrap().to_string())
        .collect()
}

fn log(sb: &Sandbox) -> Vec<Value> {
    let (ok, out, err) = sb.dam(&["log", "--json"]);
    assert!(ok, "{err}");
    let log: Value = serde_json::from_str(&out).unwrap();
    assert_matches_schema(&log, "log.schema.json");
    log["commits"].as_array().unwrap().clone()
}

#[test]
fn the_pulled_hook_edits_what_a_later_pull_brings_and_the_answer_counts_it() {
    let _guard =
        support::guard("the_pulled_hook_edits_what_a_later_pull_brings_and_the_answer_counts_it");
    let sb = hooked(FILING);
    let out = sb.output(&["pull", "--json"]);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "pulled call Bob new from fake\n"
    );
    let answer: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_matches_schema(&answer, "pull.schema.json");
    let commit = log(&sb)[0].clone();
    assert_eq!(commit["message"], "task_pulled after pull from fake");
    assert_eq!(
        commit["changes"][0]["details"]["meta"]["hook"],
        "task_pulled"
    );
    assert_eq!(
        answer["remotes"][0]["task_pulled"],
        json!({"ran": 1, "changed": 1, "failed": 0, "not_reached": 0, "commit": commit["id"]})
    );
    let mut listed = subjects(&sb);
    listed.sort();
    assert_eq!(listed, ["call Bob today", "water the plants"]);
}

#[test]
fn a_pull_says_in_plain_text_what_the_pulled_hook_did() {
    let _guard = support::guard("a_pull_says_in_plain_text_what_the_pulled_hook_did");
    let sb = hooked("function() error('no bell') end");
    let (ok, out, err) = sb.dam(&["pull"]);
    assert!(ok, "{err}");
    assert_eq!(
        out,
        "fake: 1 new, 0 updated, 0 unchanged, 0 conflicts, 0 removed upstream\n  \
         task_pulled: 1 ran, 0 changed, 1 failed, 0 not reached\n"
    );
    assert!(subjects(&sb).contains(&"call Bob".to_string()));
}

#[test]
fn a_failing_pulled_hook_warns_once_and_the_pull_is_kept() {
    let _guard = support::guard("a_failing_pulled_hook_warns_once_and_the_pull_is_kept");
    let sb = hooked("function() error('no bell') end");
    let out = sb.output(&["pull", "--json"]);
    assert!(out.status.success(), "{out:?}");
    let config = sb.config_path().display().to_string();
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        format!("warning: hook task_pulled failed at {config}:3: no bell; the pull is kept\n")
    );
    let answer: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(answer["remotes"][0]["created"], 1, "{answer}");
    assert_eq!(answer["remotes"][0]["task_pulled"]["failed"], 1, "{answer}");
}

#[test]
fn a_pull_names_in_plain_text_the_commit_its_pulled_hook_made() {
    let _guard = support::guard("a_pull_names_in_plain_text_the_commit_its_pulled_hook_made");
    let sb = hooked(FILING);
    let (ok, out, err) = sb.dam(&["pull"]);
    assert!(ok, "{err}");
    let commit = log(&sb)[0]["id"].as_str().unwrap().to_string();
    assert_eq!(
        out,
        format!(
            "fake: 1 new, 0 updated, 0 unchanged, 0 conflicts, 0 removed upstream\n  \
             task_pulled: 1 ran, 1 changed, 0 failed, 0 not reached, commit {}\n",
            &commit[..7]
        )
    );
}
