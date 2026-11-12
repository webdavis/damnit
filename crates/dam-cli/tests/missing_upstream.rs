mod support;

use serde_json::{Value, json};
use support::sandbox::Sandbox;
use support::upstream::{answer_pull_with, task};

fn listing(tasks: &[(&str, &str)], complete: bool) -> Value {
    let objects: Vec<Value> = tasks.iter().map(|(id, s)| task(id, s, None)).collect();
    json!({"objects": objects, "removed": [], "sync": null, "complete": complete})
}

fn oid_of(sb: &Sandbox, subject: &str) -> String {
    let (ok, out, err) = sb.dam(&["ls", "--json", "--no-pull"]);
    assert!(ok, "{err}");
    let listed: Value = serde_json::from_str(&out).unwrap();
    listed["objects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["subject"] == subject)
        .unwrap()["oid"]
        .as_str()
        .unwrap()
        .to_string()
}

fn pulled(sb: &Sandbox, answer: Value) {
    answer_pull_with(sb, answer);
    let (ok, _, err) = sb.dam(&["pull"]);
    assert!(ok, "{err}");
}

#[test]
fn open_tasks_a_complete_listing_leaves_out_are_noticed_until_they_come_back() {
    let _guard =
        support::guard("open_tasks_a_complete_listing_leaves_out_are_noticed_until_they_come_back");
    let sb = Sandbox::new();
    pulled(
        &sb,
        listing(&[("i:1", "milk"), ("i:2", "eggs"), ("i:3", "bread")], true),
    );
    let milk = oid_of(&sb, "milk");
    let eggs = oid_of(&sb, "eggs");
    pulled(&sb, listing(&[("i:3", "bread")], true));

    let (ok, out, err) = sb.dam(&["status"]);
    assert!(ok, "{err}");
    assert!(
        out.contains("fake no longer lists 2 open tasks; dam done <oid> if finished there, dam rm <oid> if deleted"),
        "{out}"
    );
    assert!(
        out.contains(&format!("    {} \"milk\"", &milk[..7])),
        "{out}"
    );
    assert!(
        out.contains(&format!("    {} \"eggs\"", &eggs[..7])),
        "{out}"
    );

    let status = support::status_json(&sb, &["--json"]);
    support::schema::assert_matches_schema(&status, "status.schema.json");
    let mut notices = status["notices"].clone();
    notices[0]["tasks"]
        .as_array_mut()
        .expect("the notice lists its tasks")
        .sort_by_key(|task| task["subject"].as_str().unwrap_or_default().to_string());
    assert_eq!(
        notices,
        json!([{"kind": "missing_upstream", "remote": "fake", "tasks": [
            {"oid": eggs, "subject": "eggs"}, {"oid": milk, "subject": "milk"}
        ]}])
    );
    let (ok, toon, err) = sb.dam(&["status", "--toon"]);
    assert!(ok, "{err}");
    assert!(
        toon.contains("missing_upstream") && toon.contains(&eggs),
        "{toon}"
    );

    pulled(&sb, listing(&[("i:1", "milk"), ("i:3", "bread")], true));
    let tasks = &support::status_json(&sb, &["--json"])["notices"][0]["tasks"];
    assert_eq!(tasks, &json!([{"oid": eggs, "subject": "eggs"}]));

    pulled(
        &sb,
        listing(&[("i:1", "milk"), ("i:2", "eggs"), ("i:3", "bread")], true),
    );
    assert_eq!(support::status_json(&sb, &["--json"])["notices"], json!([]));
}

#[test]
fn a_listing_that_does_not_say_it_is_complete_notices_nothing() {
    let _guard = support::guard("a_listing_that_does_not_say_it_is_complete_notices_nothing");
    let sb = Sandbox::new();
    pulled(&sb, listing(&[("i:1", "milk"), ("i:2", "eggs")], false));
    pulled(&sb, listing(&[("i:2", "eggs")], false));
    assert_eq!(support::status_json(&sb, &["--json"])["notices"], json!([]));
}

#[test]
fn a_task_done_or_changed_here_is_left_out_of_the_notice() {
    let _guard = support::guard("a_task_done_or_changed_here_is_left_out_of_the_notice");
    let sb = Sandbox::new();
    pulled(
        &sb,
        listing(&[("i:1", "milk"), ("i:2", "eggs"), ("i:3", "bread")], true),
    );
    assert!(sb.dam(&["done", &oid_of(&sb, "milk")]).0);
    assert!(
        sb.dam(&["edit", &oid_of(&sb, "eggs"), "--subject", "brown eggs"])
            .0
    );
    assert!(sb.dam(&["add", "-A"]).0);
    assert!(sb.dam(&["commit", "-m", "mine"]).0);
    pulled(&sb, listing(&[], true));
    let tasks = &support::status_json(&sb, &["--json"])["notices"][0]["tasks"];
    assert_eq!(
        tasks,
        &json!([{"oid": oid_of(&sb, "bread"), "subject": "bread"}])
    );
}
