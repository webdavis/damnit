mod support;

use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to, refused};

fn owner(sb: &Sandbox, args: &[&str], schema: &str) -> serde_json::Value {
    let mut line = args.to_vec();
    line.push("--json");
    answer(&sb.output(&line), schema)
}

fn seeded_with_rent() -> (Sandbox, String, String) {
    let sb = Sandbox::new();
    let rent = sb.new_object(&["Pay rent"]);
    owner(&sb, &["add", "-A"], "add.schema.json");
    owner(&sb, &["commit", "-m", "seed"], "commit.schema.json");
    let held = grant_to(
        &sb,
        "vpt",
        1,
        &grant("\"task:read\", \"task:create\", \"task:change\""),
    );
    (sb, held, rent)
}

fn send(sb: &Sandbox, held: &str, args: &[&str]) -> String {
    let mut line = args.to_vec();
    line.push("--json");
    let sent = answer(&sb.output_holding(held, &line), "sent.schema.json");
    sent["hash"].as_str().unwrap().to_string()
}

#[test]
fn reset_sends_a_tokens_added_change_back_to_waiting_and_keeps_the_owners_edit() {
    let _guard = support::guard(
        "reset_sends_a_tokens_added_change_back_to_waiting_and_keeps_the_owners_edit",
    );
    let (sb, held, rent) = seeded_with_rent();
    let retitle = send(&sb, &held, &["edit", &rent, "--subject", "Pay the rent"]);
    let (ok, _, err) = sb.dam(&["edit", &rent, "--body", "by the first"]);
    assert!(ok, "{err}");
    owner(&sb, &["add", "-A"], "add.schema.json");

    let (ok, _, err) = sb.dam(&["reset", &rent]);
    assert!(ok, "{err}");

    let status = owner(&sb, &["status"], "status.schema.json");
    assert_eq!(status["staged"], serde_json::json!([]));
    assert_eq!(status["waiting"][0]["hash"], retitle.as_str());
    assert_eq!(status["unstaged"][0]["subject"], "Pay rent");
    assert_eq!(status["unstaged"][0]["fields"], serde_json::json!(["body"]));
    assert_eq!(status["unstaged"][0]["from"], serde_json::Value::Null);

    owner(&sb, &["add", "-A"], "add.schema.json");
    let commit = owner(&sb, &["commit", "-m", "both"], "commit.schema.json");
    let from: Vec<&serde_json::Value> = commit["changes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| &c["from"])
        .collect();
    assert!(from.contains(&&serde_json::json!("vpt")), "{commit}");
}

#[test]
fn reset_of_everything_sends_a_tokens_new_object_back_to_waiting() {
    let _guard = support::guard("reset_of_everything_sends_a_tokens_new_object_back_to_waiting");
    let (sb, held, _) = seeded_with_rent();
    let dentist = send(&sb, &held, &["new", "Call the dentist"]);
    owner(&sb, &["add", &dentist[..7]], "add.schema.json");

    let (ok, _, err) = sb.dam(&["reset"]);
    assert!(ok, "{err}");

    let status = owner(&sb, &["status"], "status.schema.json");
    assert_eq!(status["staged"], serde_json::json!([]));
    assert_eq!(status["unstaged"], serde_json::json!([]));
    assert_eq!(status["waiting"][0]["hash"], dentist.as_str());

    owner(&sb, &["add", &dentist[..7]], "add.schema.json");
    let commit = owner(&sb, &["commit", "-m", "dentist"], "commit.schema.json");
    assert_eq!(commit["changes"][0]["from"], "vpt");
}

#[test]
fn reset_refuses_to_drop_the_owners_edit_of_a_tokens_new_object() {
    let _guard = support::guard("reset_refuses_to_drop_the_owners_edit_of_a_tokens_new_object");
    let (sb, held, _) = seeded_with_rent();
    let dentist = send(&sb, &held, &["new", "Call the dentist"]);
    owner(&sb, &["add", &dentist[..7]], "add.schema.json");
    let (ok, _, err) = sb.dam(&["edit", &dentist[..7], "--body", "before noon"]);
    assert!(ok, "{err}");

    assert_eq!(
        refused(&sb.output(&["reset", "--json"])),
        "item_edited_since_added"
    );
    let status = owner(&sb, &["status"], "status.schema.json");
    assert_eq!(status["staged"][0]["from"], "vpt");
    assert_eq!(status["waiting"], serde_json::json!([]));
}
