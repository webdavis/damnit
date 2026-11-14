mod support;

use serde_json::{Value, json};
use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to, refused};

const RIGHTS: [&str; 4] = ["task:create", "task:read", "task:change", "task:remove"];

fn owner(sb: &Sandbox, args: &[&str]) {
    let (ok, _, err) = sb.dam(args);
    assert!(ok, "{args:?}: {err}");
}

fn sent(sb: &Sandbox, held: &str, line: &[&str]) -> Value {
    let mut line = line.to_vec();
    line.push("--json");
    let out = sb.output_holding(held, &line);
    assert!(out.status.success(), "{out:?}");
    serde_json::from_slice(&out.stdout).unwrap()
}

fn item_of_kind(kind: &str) -> (Sandbox, String, String, Vec<String>) {
    let sb = Sandbox::new();
    let held = grant_to(&sb, "tool", 1, vec![grant(&RIGHTS)]);
    let new = ["new", "Call the dentist"];
    let oid = sent(&sb, &held, &new)["oid"].as_str().unwrap().to_string();
    let line: Vec<String> = match kind {
        "new" => return (sb, held, oid, new.map(String::from).to_vec()),
        "change" => vec![
            "edit".into(),
            oid.clone(),
            "--subject".into(),
            "Call".into(),
        ],
        _ => vec!["rm".into(), oid.clone()],
    };
    owner(&sb, &["add", &oid]);
    owner(&sb, &["commit", "-m", "dentist"]);
    let line_refs: Vec<&str> = line.iter().map(String::as_str).collect();
    let answer = sent(&sb, &held, &line_refs);
    let hash = answer.get("hash").or(answer.get("item")).unwrap();
    (sb, held, hash.as_str().unwrap().to_string(), line)
}

fn waiting_hashes(sb: &Sandbox) -> Vec<Value> {
    let status = support::status_json(sb, &["--json"]);
    status["waiting"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["hash"].clone())
        .collect()
}

#[test]
fn reset_puts_a_declined_item_of_each_kind_back_to_waiting() {
    let _guard = support::guard("reset_puts_a_declined_item_of_each_kind_back_to_waiting");
    for (kind, named) in [("new", 40), ("change", 7), ("removal", 40)] {
        let (sb, held, hash, line) = item_of_kind(kind);
        owner(&sb, &["rm", &hash]);
        let reset = answer(
            &sb.output(&["reset", &hash[..named], "--json"]),
            "reset.schema.json",
        );
        assert_eq!(reset, json!({"unstaged": [], "waiting": [hash]}), "{kind}");
        assert_eq!(waiting_hashes(&sb), vec![json!(hash)], "{kind}");
        let line: Vec<&str> = line.iter().map(String::as_str).collect();
        let again = sent(&sb, &held, &line);
        let named_again = again.get("hash").or(again.get("item")).unwrap();
        assert_eq!(named_again, &json!(hash), "{kind}");
        owner(&sb, &["add", &hash]);
    }
}

#[test]
fn the_text_says_the_item_waits_again() {
    let _guard = support::guard("the_text_says_the_item_waits_again");
    let (sb, _, hash, _) = item_of_kind("change");
    owner(&sb, &["rm", &hash]);
    let (ok, out, err) = sb.dam(&["reset", &hash]);
    assert!(ok, "{err}");
    assert_eq!(out.trim_end(), format!("waiting again {}", &hash[..7]));
}

#[test]
fn reset_of_a_withdrawn_item_is_refused_and_leaves_it_withdrawn() {
    let _guard = support::guard("reset_of_a_withdrawn_item_is_refused_and_leaves_it_withdrawn");
    let (sb, held, hash, _) = item_of_kind("change");
    let out = sb.output_holding(&held, &["rm", &hash]);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(
        refused(&sb.output(&["reset", &hash, "--json"])),
        "item_not_waiting"
    );
    assert!(waiting_hashes(&sb).is_empty());
}

#[test]
fn reset_of_an_added_change_or_removal_by_its_hash_sends_it_back_to_waiting() {
    let _guard =
        support::guard("reset_of_an_added_change_or_removal_by_its_hash_sends_it_back_to_waiting");
    for kind in ["change", "removal"] {
        let (sb, _, hash, _) = item_of_kind(kind);
        owner(&sb, &["add", &hash]);
        owner(&sb, &["reset", &hash]);
        assert_eq!(waiting_hashes(&sb), vec![json!(hash)], "{kind}");
        let status = support::status_json(&sb, &["--json"]);
        assert_eq!(status["staged"], json!([]), "{kind}");
    }
}
