use crate::support;

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

fn commit_everything(sb: &Sandbox, message: &str) {
    assert!(sb.dam(&["add", "-A"]).0);
    assert!(sb.dam(&["commit", "-m", message]).0);
}

fn unstaged_fields(sb: &Sandbox) -> serde_json::Value {
    status_json(sb, &["-o", "json"])["unstaged"][0]["fields"].clone()
}
