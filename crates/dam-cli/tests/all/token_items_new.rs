use crate::support;

use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to, token};

fn owner_objects(sb: &Sandbox) -> serde_json::Value {
    let (ok, out, err) = sb.dam(&["ls", "--json"]);
    assert!(ok, "{err}");
    serde_json::from_str::<serde_json::Value>(&out).unwrap()["objects"].clone()
}

#[test]
fn a_tokens_new_task_answers_new_with_the_oid_it_will_have_and_waits() {
    let _guard =
        support::guard("a_tokens_new_task_answers_new_with_the_oid_it_will_have_and_waits");
    let sb = Sandbox::new();
    let held = grant_to(&sb, "tool", 1, vec![grant(&["task:create"])]);
    let line = ["new", "Call the dentist", "--path", "inbox", "--json"];
    let sent = answer(&sb.output_holding(&held, &line), "sent.schema.json");
    assert_eq!(sent["answer"], "new");
    assert_eq!(sent["item"], "new");
    assert_eq!(sent["oid"], sent["hash"]);
    assert_eq!(sent["subject"], "Call the dentist");
    assert_eq!(sent["path"], "inbox/");
    assert_eq!(sent["from"], "tool");
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
