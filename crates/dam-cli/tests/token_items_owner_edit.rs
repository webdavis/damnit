mod support;

use serde_json::{Value, json};
use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to};

struct Rent {
    sb: Sandbox,
    rent: String,
    hash: String,
}

fn owner(sb: &Sandbox, args: &[&str], schema: &str) -> Value {
    let mut line = args.to_vec();
    line.push("--json");
    answer(&sb.output(&line), schema)
}

fn retitled_by_vpt_and_made_to_wait_on_keys_by_the_owner() -> Rent {
    let sb = Sandbox::new();
    let rent = sb.new_object(&["Pay rent"]);
    let keys = sb.new_object(&["Find the keys"]);
    owner(&sb, &["add", "-A"], "add.schema.json");
    owner(&sb, &["commit", "-m", "seed"], "commit.schema.json");
    let held = grant_to(&sb, "vpt", 1, &grant("\"task:read\", \"task:change\""));
    let line = ["edit", &rent, "--subject", "Pay the rent", "--json"];
    let sent = answer(&sb.output_holding(&held, &line), "sent.schema.json");
    let (ok, _, err) = sb.dam(&["edit", &rent, "--depends", &keys]);
    assert!(ok, "{err}");
    let hash = sent["hash"].as_str().unwrap().to_string();
    Rent { sb, rent, hash }
}

fn committed_under_each_author(sb: &Sandbox) {
    let commit = owner(sb, &["commit", "-m", "both"], "commit.schema.json");
    let changes = commit["changes"].as_array().unwrap();
    assert_eq!(changes.len(), 2, "{commit}");
    assert_eq!(changes[0]["from"], "vpt");
    assert_eq!(changes[0]["fields"], json!(["subject"]));
    assert_eq!(changes[1]["from"], Value::Null);
    assert_eq!(changes[1]["fields"], json!(["depends"]));
    let status = owner(sb, &["status"], "status.schema.json");
    assert_eq!(status["unstaged"], json!([]), "{status}");
    assert_eq!(status["staged"], json!([]), "{status}");
}

#[test]
fn add_all_stages_a_tokens_change_and_the_owners_edit_of_one_task_each_under_its_author() {
    let _guard = support::guard(
        "add_all_stages_a_tokens_change_and_the_owners_edit_of_one_task_each_under_its_author",
    );
    let r = retitled_by_vpt_and_made_to_wait_on_keys_by_the_owner();
    let added = owner(&r.sb, &["add", "-A"], "add.schema.json");
    assert_eq!(added["staged"].as_array().unwrap().len(), 2, "{added}");
    committed_under_each_author(&r.sb);
}

#[test]
fn adding_the_task_after_its_item_keeps_the_tokens_change_under_the_token() {
    let _guard =
        support::guard("adding_the_task_after_its_item_keeps_the_tokens_change_under_the_token");
    let r = retitled_by_vpt_and_made_to_wait_on_keys_by_the_owner();
    owner(&r.sb, &["add", &r.hash], "add.schema.json");
    owner(&r.sb, &["add", &r.rent], "add.schema.json");
    committed_under_each_author(&r.sb);
}
