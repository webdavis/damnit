mod support;

use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to, refused};

fn added_from_tool(allow: &[&str]) -> (Sandbox, String, String) {
    let sb = Sandbox::new();
    let held = grant_to(&sb, "tool", 1, vec![grant(allow)]);
    let line = ["new", "Call the dentist", "--json"];
    let sent = answer(&sb.output_holding(&held, &line), "sent.schema.json");
    let oid = sent["oid"].as_str().unwrap().to_string();
    for step in [&["add", &oid][..], &["commit", "-m", "dentist"]] {
        let (ok, _, err) = sb.dam(step);
        assert!(ok, "{err}");
    }
    (sb, held, oid)
}

fn removal(sb: &Sandbox, held: &str, oid: &str) -> serde_json::Value {
    answer(
        &sb.output_holding(held, &["rm", oid, "--json"]),
        "rm.schema.json",
    )
}

fn listed(sb: &Sandbox) -> usize {
    let (ok, out, err) = sb.dam(&["ls", "--json", "--no-pull"]);
    assert!(ok, "{err}");
    let ls: serde_json::Value = serde_json::from_str(&out).unwrap();
    ls["objects"].as_array().unwrap().len()
}

const EVERY_RIGHT: [&str; 3] = ["task:create", "task:read", "task:remove"];

#[test]
fn a_tokens_rm_of_an_added_task_waits_once_until_the_owner_adds_it() {
    let _guard = support::guard("a_tokens_rm_of_an_added_task_waits_once_until_the_owner_adds_it");
    let (sb, held, oid) = added_from_tool(&EVERY_RIGHT);
    let first = removal(&sb, &held, &oid);
    assert_eq!(first["answer"], "waiting");
    assert_eq!(first["oid"], oid.as_str());
    assert_eq!(removal(&sb, &held, &oid), first);
    let status = support::status_json(&sb, &["--json"]);
    let waiting = status["waiting"].as_array().unwrap();
    assert_eq!(waiting.len(), 1);
    assert_eq!(waiting[0]["hash"], first["item"]);
    assert_eq!(waiting[0]["item"], "removal");
    assert_eq!(waiting[0]["from"], "tool");
    assert_eq!(listed(&sb), 1);
    let (ok, _, err) = sb.dam(&["add", first["item"].as_str().unwrap()]);
    assert!(ok, "{err}");
    assert_eq!(listed(&sb), 0);
    let status = support::status_json(&sb, &["--json"]);
    assert_eq!(status["staged"][0]["op"], "delete");
    assert_eq!(status["staged"][0]["from"], "tool");
}

#[test]
fn a_removal_the_owner_declined_answers_declined_and_the_task_stays() {
    let _guard = support::guard("a_removal_the_owner_declined_answers_declined_and_the_task_stays");
    let (sb, held, oid) = added_from_tool(&EVERY_RIGHT);
    let first = removal(&sb, &held, &oid);
    let (ok, _, err) = sb.dam(&["rm", first["item"].as_str().unwrap()]);
    assert!(ok, "{err}");
    let again = removal(&sb, &held, &oid);
    assert_eq!(again["answer"], "declined");
    assert_eq!(again["item"], first["item"]);
    assert_eq!(listed(&sb), 1);
    assert!(
        support::status_json(&sb, &["--json"])["waiting"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn removing_an_added_task_needs_task_remove() {
    let _guard = support::guard("removing_an_added_task_needs_task_remove");
    let (sb, held, oid) = added_from_tool(&["task:create", "task:read"]);
    let out = sb.output_holding(&held, &["rm", &oid, "--json"]);
    assert_eq!(refused(&out), "not_permitted");
    assert_eq!(listed(&sb), 1);
}

#[test]
fn the_text_answer_is_one_plain_line() {
    let _guard = support::guard("the_text_answer_is_one_plain_line");
    let (sb, held, oid) = added_from_tool(&EVERY_RIGHT);
    let out = sb.output_holding(&held, &["rm", &oid]);
    assert!(out.status.success(), "{out:?}");
    let text = String::from_utf8_lossy(&out.stdout);
    let again = removal(&sb, &held, &oid);
    let item = &again["item"].as_str().unwrap()[..7];
    assert_eq!(text.trim_end(), format!("waiting  {item}  {}", &oid[..7]));
}
