use crate::support;

use serde_json::{Value, json};
use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to};

const LABELLING: &[(&str, &str)] = &[
    ("task_new", "function() return { labels = { 'new' } } end"),
    (
        "task_changed",
        "function() return { labels = { 'changed' } } end",
    ),
    ("task_done", "function() return { labels = { 'done' } } end"),
];

fn hooked(sb: &Sandbox, hooks: &[(&str, &str)]) {
    sb.write_hooks(hooks);
    sb.write_secrets_lua("return { schema = 1 }\n");
}

fn created(sb: &Sandbox) -> Value {
    answer(
        &sb.output(&["new", "buy milk", "-o", "json"]),
        "new.schema.json",
    )
}

#[test]
fn new_answers_with_the_task_as_its_hook_left_it_and_names_the_hook() {
    let _guard = support::guard("new_answers_with_the_task_as_its_hook_left_it_and_names_the_hook");
    let sb = Sandbox::new();
    hooked(&sb, LABELLING);
    let task = created(&sb);
    assert_eq!(task["labels"], json!(["new"]));
    assert_eq!(
        task["changed_by_hook"],
        json!([{ "oid": task["oid"], "hook": "task_new" }])
    );
}

#[test]
fn an_answer_carries_no_changed_by_hook_when_the_hook_changed_nothing() {
    let _guard =
        support::guard("an_answer_carries_no_changed_by_hook_when_the_hook_changed_nothing");
    let sb = Sandbox::new();
    hooked(&sb, &[("task_new", "function() end")]);
    let task = created(&sb);
    assert!(task.get("changed_by_hook").is_none(), "{task}");
}

#[test]
fn edit_and_done_name_the_hook_that_changed_the_task() {
    let _guard = support::guard("edit_and_done_name_the_hook_that_changed_the_task");
    let sb = Sandbox::new();
    hooked(&sb, LABELLING);
    let oid = created(&sb)["oid"].as_str().unwrap().to_string();
    let edited = answer(
        &sb.output(&["edit", &oid, "--subject", "buy oat milk", "-o", "json"]),
        "edit.schema.json",
    );
    assert_eq!(edited["labels"], json!(["changed", "new"]));
    assert_eq!(
        edited["changed_by_hook"],
        json!([{ "oid": oid, "hook": "task_changed" }])
    );
    let done = answer(
        &sb.output(&["done", &oid, "-o", "json"]),
        "done.schema.json",
    );
    assert_eq!(
        done["changed_by_hook"],
        json!([{ "oid": oid, "hook": "task_done" }])
    );
}

#[test]
fn done_names_every_task_a_hook_changed_not_only_the_one_completed() {
    let _guard = support::guard("done_names_every_task_a_hook_changed_not_only_the_one_completed");
    let sb = Sandbox::new();
    let parent = sb.new_object(&["parent"]);
    let child = sb.new_object(&["child", "--path", "parent/"]);
    hooked(&sb, LABELLING);
    let done = answer(
        &sb.output(&["done", &parent, "--force", "--children", "up", "-o", "json"]),
        "done.schema.json",
    );
    let mut changed: Vec<(String, String)> = done["changed_by_hook"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|change| {
            let oid = change["oid"].as_str().unwrap_or_default();
            let hook = change["hook"].as_str().unwrap_or_default();
            (
                hook.to_owned(),
                oid[..child.len().min(oid.len())].to_owned(),
            )
        })
        .collect();
    changed.sort();
    assert_eq!(
        changed,
        [
            ("task_changed".to_owned(), child),
            ("task_done".to_owned(), parent),
        ]
    );
}

#[test]
fn add_names_each_task_a_hook_changed_as_it_took_an_item() {
    let _guard = support::guard("add_names_each_task_a_hook_changed_as_it_took_an_item");
    let sb = Sandbox::new();
    let held = grant_to(&sb, "tool", 1, vec![grant(&["task:read", "task:create"])]);
    let sent = answer(
        &sb.output_holding(&held, &["new", "call the dentist", "-o", "json"]),
        "sent.schema.json",
    );
    hooked(&sb, LABELLING);
    let hash = sent["hash"].as_str().unwrap();
    let added = answer(&sb.output(&["add", hash, "-o", "json"]), "add.schema.json");
    assert_eq!(
        added["changed_by_hook"],
        json!([{ "oid": sent["oid"], "hook": "task_new" }])
    );
}

#[test]
fn plain_text_says_which_hook_changed_the_task() {
    let _guard = support::guard("plain_text_says_which_hook_changed_the_task");
    let sb = Sandbox::new();
    hooked(&sb, LABELLING);
    let out = sb.output(&["new", "buy milk"]);
    assert!(out.status.success(), "{out:?}");
    let listed: Value = serde_json::from_slice(&sb.output(&["list", "-o", "json"]).stdout).unwrap();
    let oid = listed["objects"][0]["oid"].as_str().unwrap();
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        format!("dam: hook task_new changed {}\n", &oid[..7])
    );
}
