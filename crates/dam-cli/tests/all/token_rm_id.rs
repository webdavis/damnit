use crate::support;

use support::request::by_request;
use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to};

fn tool(allow: &[&str]) -> (Sandbox, String) {
    let sb = Sandbox::new();
    let held = grant_to(&sb, "tool", 1, vec![grant(allow)]);
    (sb, held)
}

fn send(sb: &Sandbox, held: &str, session: &str) -> String {
    let meta = format!("session={session}");
    let line = [
        "new",
        "Fix the build",
        "--id",
        "build",
        "--meta",
        &meta,
        "-o",
        "json",
    ];
    let sent = answer(&sb.output_holding(held, &line), "sent.schema.json");
    sent["oid"].as_str().unwrap().to_string()
}

fn take_back(sb: &Sandbox, held: &str, session: &str) -> serde_json::Value {
    let meta = format!("session={session}");
    let line = ["rm", "--id", "build", "--meta", &meta, "-o", "json"];
    answer(&sb.output_holding(held, &line), "rm.schema.json")
}

#[test]
fn a_waiting_item_is_withdrawn_by_its_id_and_a_repeat_hears_nothing() {
    let _guard = support::guard("a_waiting_item_is_withdrawn_by_its_id_and_a_repeat_hears_nothing");
    let (sb, held) = tool(&["task:create"]);
    let oid = send(&sb, &held, "a");
    let withdrawn = take_back(&sb, &held, "a");
    assert_eq!(withdrawn["answer"], "withdrawn");
    assert_eq!(withdrawn["item"], oid.as_str());
    assert_eq!(withdrawn["oid"], oid.as_str());
    let again = take_back(&sb, &held, "a");
    assert_eq!(
        again,
        serde_json::json!({"answer": "nothing", "item": null, "oid": null})
    );
}

#[test]
fn an_oid_and_an_id_together_or_neither_is_a_usage_error() {
    let _guard = support::guard("an_oid_and_an_id_together_or_neither_is_a_usage_error");
    let sb = Sandbox::new();
    let oid = sb.new_object(&["Fix the build"]);
    for (line, problem, word) in [
        (
            &["rm", &oid, "--id", "build", "-o", "json"][..],
            "conflict",
            serde_json::json!("--id"),
        ),
        (
            &["rm", "-o", "json"],
            "missing_argument",
            serde_json::Value::Null,
        ),
    ] {
        let out = sb.output(line);
        assert_eq!(out.status.code(), Some(2), "{line:?}");
        let usage = &support::errors::error_document(&out)["error"]["usage"];
        assert_eq!(
            usage,
            &serde_json::json!({"problem": problem, "word": word}),
            "{line:?}"
        );
    }
    let out = by_request(&sb, &["rm"], &serde_json::json!({"id": "build"}));
    assert_eq!(answer(&out, "rm.schema.json")["answer"], "nothing");
}
