mod support;

use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to, refused, token};

fn owner_objects(sb: &Sandbox) -> serde_json::Value {
    let (ok, out, err) = sb.dam(&["ls", "--json", "--no-pull"]);
    assert!(ok, "{err}");
    serde_json::from_str::<serde_json::Value>(&out).unwrap()["objects"].clone()
}

#[test]
fn a_tokens_new_task_answers_new_with_the_oid_it_will_have_and_waits() {
    let _guard =
        support::guard("a_tokens_new_task_answers_new_with_the_oid_it_will_have_and_waits");
    let sb = Sandbox::new();
    let held = grant_to(&sb, "vpt", 1, &grant("\"task:create\""));
    let line = ["new", "Call the dentist", "--path", "inbox", "--json"];
    let sent = answer(&sb.output_holding(&held, &line), "sent.schema.json");
    assert_eq!(sent["answer"], "new");
    assert_eq!(sent["item"], "new");
    assert_eq!(sent["oid"], sent["hash"]);
    assert_eq!(sent["subject"], "Call the dentist");
    assert_eq!(sent["path"], "inbox/");
    assert_eq!(sent["from"], "vpt");
    assert_eq!(sent["key_id"], token(1).key_id().as_str());
    assert_eq!(sent["channel"], "local");
    assert_eq!(owner_objects(&sb), serde_json::json!([]));

    let text = sb.output_holding(&held, &line[..4]);
    let hash = sent["hash"].as_str().unwrap();
    assert_eq!(
        String::from_utf8_lossy(&text.stdout).trim(),
        format!("duplicate  {}  Call the dentist", &hash[..7])
    );
    let toon = sb.output_holding(
        &held,
        &["new", "Call the dentist", "--path", "inbox", "--toon"],
    );
    assert!(String::from_utf8_lossy(&toon.stdout).contains(hash));
}

#[test]
fn an_identical_resend_answers_duplicate_with_the_same_hash() {
    let _guard = support::guard("an_identical_resend_answers_duplicate_with_the_same_hash");
    let sb = Sandbox::new();
    let held = grant_to(&sb, "vpt", 1, &grant("\"task:create\""));
    let line = ["new", "Call the dentist", "--json"];
    let first = answer(&sb.output_holding(&held, &line), "sent.schema.json");
    let again = answer(&sb.output_holding(&held, &line), "sent.schema.json");
    assert_eq!(again["answer"], "duplicate");
    assert_eq!(again["hash"], first["hash"]);
}

#[test]
fn a_new_task_outside_the_grant_is_refused_and_nothing_waits() {
    let _guard = support::guard("a_new_task_outside_the_grant_is_refused_and_nothing_waits");
    let sb = Sandbox::new();
    let held = grant_to(
        &sb,
        "vpt",
        1,
        &format!("{}paths = [\"work\"]\n", grant("\"task:create\"")),
    );
    let home = ["new", "laundry", "--path", "home", "--json"];
    assert_eq!(refused(&sb.output_holding(&held, &home)), "not_permitted");
    let event = [
        "new",
        "--event",
        "standup",
        "--path",
        "work",
        "--start",
        "2026-11-12T09:00",
        "--end",
        "2026-11-12T09:15",
        "--json",
    ];
    assert_eq!(refused(&sb.output_holding(&held, &event)), "not_permitted");
    let work = ["new", "report", "--path", "work", "--json"];
    answer(&sb.output_holding(&held, &work), "sent.schema.json");
}
