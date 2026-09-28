mod support;

use support::sandbox::Sandbox;
use support::status_json;

#[test]
fn a_change_document_names_the_fields_it_touches() {
    let _guard = support::guard("a_change_document_names_the_fields_it_touches");
    let sb = Sandbox::new();
    let oid = sb.new_object(&["buy oat milk", "--due", "2026-09-25", "-p", "1"]);
    assert_eq!(
        unstaged_fields(&sb),
        serde_json::json!(["subject", "priority", "due"]),
        "a create names every field it sets"
    );

    commit_everything(&sb, "the create");
    assert!(sb.dam(&["edit", &oid, "--subject", "buy soy milk"]).0);
    assert_eq!(unstaged_fields(&sb), serde_json::json!(["subject"]));

    commit_everything(&sb, "the edit");
    assert!(sb.dam(&["done", &oid]).0);
    assert_eq!(unstaged_fields(&sb), serde_json::json!(["done"]));

    commit_everything(&sb, "the completion");
    assert!(sb.dam(&["edit", &oid, "--undone"]).0);
    assert_eq!(
        unstaged_fields(&sb),
        serde_json::json!(["done"]),
        "reopening moves done and nothing else"
    );

    assert!(sb.dam(&["rm", &oid]).0);
    assert_eq!(
        unstaged_fields(&sb),
        serde_json::json!([]),
        "a delete removes the object whole and names no field"
    );
}

#[test]
fn an_unpushed_row_names_its_commits_in_log_order() {
    let _guard = support::guard("an_unpushed_row_names_its_commits_in_log_order");
    let sb = Sandbox::new();
    for (subject, message) in [("first", "one"), ("second", "two")] {
        sb.new_object(&[subject]);
        assert!(sb.dam(&["add", "-A"]).0);
        assert!(sb.dam(&["commit", "-m", message]).0);
    }

    let (ok, out, err) = sb.dam(&["log", "--json"]);
    assert!(ok, "{err}");
    let log: serde_json::Value = serde_json::from_str(&out).unwrap();
    let logged: Vec<&str> = log["commits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["id"].as_str().unwrap())
        .collect();
    assert_eq!(logged.len(), 2);

    let row = status_json(&sb, &["--json"])["unpushed"][0].clone();
    assert_eq!(row["remote"], "fake");
    assert_eq!(
        row["commit_ids"],
        serde_json::json!(logged),
        "the commit ids read in the order dam log lists them"
    );
    assert_eq!(
        row["commits"], 2,
        "the count is the length of the list beside it"
    );
}

#[test]
fn an_unpushed_row_names_each_object_its_commits_touch_once() {
    let _guard = support::guard("an_unpushed_row_names_each_object_its_commits_touch_once");
    let sb = Sandbox::new();
    let first = sb.new_object(&["first"]);
    assert!(sb.dam(&["add", "-A"]).0);
    assert!(sb.dam(&["commit", "-m", "one"]).0);
    let second = sb.new_object(&["second"]);
    assert!(sb.dam(&["add", "-A"]).0);
    assert!(sb.dam(&["commit", "-m", "two"]).0);
    assert!(sb.dam(&["edit", &second, "--priority", "1"]).0);
    assert!(sb.dam(&["add", "-A"]).0);
    assert!(sb.dam(&["commit", "-m", "three"]).0);

    let row = status_json(&sb, &["--json"])["unpushed"][0].clone();
    assert_eq!(row["commits"], 3);
    let oids: Vec<String> = row["oids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        oids.len(),
        2,
        "the object two commits touched is named once: {oids:?}"
    );
    assert!(
        oids[0].starts_with(&second),
        "the object the newest commit touched comes first: {oids:?}"
    );
    assert!(
        oids[1].starts_with(&first),
        "then the one only an older commit touched: {oids:?}"
    );
}

#[test]
fn status_embeds_no_object_until_asked_for_one() {
    let _guard = support::guard("status_embeds_no_object_until_asked_for_one");
    let sb = Sandbox::new();
    let oid = sb.new_object(&[
        "buy oat milk",
        "--due",
        "2026-09-25",
        "-p",
        "1",
        "--label",
        "errand",
    ]);

    let row = status_json(&sb, &["--json"])["unstaged"][0].clone();
    assert!(
        row.get("before").is_none() && row.get("after").is_none(),
        "{row}"
    );
    assert!(row["oid"].as_str().unwrap().starts_with(&oid));
    assert_eq!(row["op"], "create");
    assert_eq!(row["kind"], "task");
    assert_eq!(row["subject"], "buy oat milk");
    assert_eq!(row["due"], "2026-09-25");
    assert_eq!(row["priority"], 1);
    assert_eq!(row["done"], false);
    assert_eq!(row["labels"], serde_json::json!(["errand"]));

    let full = status_json(&sb, &["--json", "--full"])["unstaged"][0].clone();
    assert_eq!(full["before"], serde_json::Value::Null);
    assert_eq!(full["after"]["subject"], "buy oat milk");
    assert_eq!(full["after"]["body"], "");
    for key in ["oid", "op", "fields", "subject", "due"] {
        assert_eq!(full[key], row[key], "--full dropped {key}");
    }
}

fn commit_everything(sb: &Sandbox, message: &str) {
    assert!(sb.dam(&["add", "-A"]).0);
    assert!(sb.dam(&["commit", "-m", message]).0);
}

fn unstaged_fields(sb: &Sandbox) -> serde_json::Value {
    status_json(sb, &["--json"])["unstaged"][0]["fields"].clone()
}
