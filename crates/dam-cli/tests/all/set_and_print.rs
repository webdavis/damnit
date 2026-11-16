use crate::support;

use serde_json::{Value, json};
use support::sandbox::Sandbox;

fn with_effort() -> Sandbox {
    let sb = Sandbox::new();
    sb.add_category("effort", &["light", "deep"], true);
    sb
}

fn shown(sb: &Sandbox, oid: &str) -> Value {
    let (ok, out, err) = sb.dam(&["show", oid, "-o", "json"]);
    assert!(ok, "{err}");
    serde_json::from_str(&out).unwrap()
}

#[test]
fn new_sets_a_recurrence_and_a_category_label() {
    let _guard = support::guard("new_sets_a_recurrence_and_a_category_label");
    let sb = with_effort();
    let oid = sb.new_object(&[
        "water the plants",
        "--recurrence",
        "every week",
        "--category",
        "effort=light",
    ]);
    let task = shown(&sb, &oid);
    assert_eq!(task["recurrence"], "every week");
    assert_eq!(task["labels"], json!(["light"]));
    let (ok, _, err) = sb.dam(&["new", "x", "--category", "effort=huge"]);
    assert!(!ok);
    assert!(err.contains("light, deep"), "{err}");
}

#[test]
fn list_rows_carry_category_and_overdue() {
    let _guard = support::guard("list_rows_carry_category_and_overdue");
    let sb = with_effort();
    sb.new_object(&["late", "--due", "2020-01-01", "--category", "effort=deep"]);
    let (ok, out, err) = sb.dam(&["list", "-o", "json"]);
    assert!(ok, "{err}");
    let row = &serde_json::from_str::<Value>(&out).unwrap()["objects"][0];
    assert_eq!(row["category"], json!({"effort": "deep"}));
    assert_eq!(row["overdue"], true);
    assert_eq!(row["blocked"], false);
}
