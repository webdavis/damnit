mod support;

use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to, refused};

fn sent_by_a(rights_of_b: &[&str]) -> (Sandbox, String, String) {
    let sb = Sandbox::new();
    let a = grant_to(&sb, "a", 1, vec![grant(&["task:create", "task:remove"])]);
    let b = grant_to(&sb, "b", 2, vec![grant(rights_of_b)]);
    let line = ["new", "Fix the build", "--id", "build", "--json"];
    let sent = answer(&sb.output_holding(&a, &line), "sent.schema.json");
    (sb, b, sent["oid"].as_str().unwrap().to_string())
}

fn still_waiting(sb: &Sandbox, oid: &str) {
    let status = support::status_json(sb, &["--json"]);
    assert_eq!(status["waiting"][0]["hash"], oid, "{status}");
}

fn b_cannot_take_back_what_a_sent(rights_of_b: &[&str]) {
    let (sb, b, oid) = sent_by_a(rights_of_b);
    let by_id = sb.output_holding(&b, &["rm", "--id", "build", "--json"]);
    assert_eq!(answer(&by_id, "rm.schema.json")["answer"], "nothing");
    let by_oid = sb.output_holding(&b, &["rm", &oid, "--json"]);
    assert_eq!(refused(&by_oid), "no_such_object");
    let by_prefix = sb.output_holding(&b, &["rm", &oid[..7], "--json"]);
    assert_eq!(refused(&by_prefix), "no_working_object");
    still_waiting(&sb, &oid);
}

#[test]
fn another_token_cannot_take_back_an_item_by_its_id_oid_or_prefix() {
    let _guard = support::guard("another_token_cannot_take_back_an_item_by_its_id_oid_or_prefix");
    b_cannot_take_back_what_a_sent(&["task:create", "task:read", "task:remove"]);
}

#[test]
fn another_token_without_task_remove_cannot_take_back_an_item_either() {
    let _guard =
        support::guard("another_token_without_task_remove_cannot_take_back_an_item_either");
    b_cannot_take_back_what_a_sent(&["task:create"]);
}
