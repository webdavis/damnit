mod support;

use std::io::Write;

use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;

const OID: &str = "0123456789abcdef0123456789abcdef01234567";

fn fed(sb: &Sandbox, args: &[&str], input: &str) -> std::process::Output {
    let mut child = sb.spawn_with_stdin(args, std::process::Stdio::piped());
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn json(bytes: &[u8]) -> serde_json::Value {
    serde_json::from_slice(bytes)
        .unwrap_or_else(|e| panic!("{e}: {}", String::from_utf8_lossy(bytes)))
}

fn closed_day() -> serde_json::Value {
    serde_json::json!({
        "day": "2026-11-11",
        "plan": [OID],
        "proposals": [OID],
        "decisions": ["move the dentist to Friday"],
        "tasks": [{"oid": OID, "stalls": 2, "carried_forward": true}],
        "closed_at": "2026-11-11T22:00:00Z"
    })
}

#[test]
fn a_day_written_as_json_reads_back_as_json_and_toon_and_lists_as_closed() {
    let _guard =
        support::guard("a_day_written_as_json_reads_back_as_json_and_toon_and_lists_as_closed");
    let sb = Sandbox::new();
    let written = fed(&sb, &["day", "set", "--json"], &closed_day().to_string());
    assert!(written.status.success(), "{written:?}");
    assert_eq!(json(&written.stdout), closed_day());

    let shown = sb.output(&["day", "show", "2026-11-11", "--json"]);
    assert_eq!(json(&shown.stdout), closed_day());
    assert_matches_schema(&json(&shown.stdout), "day.schema.json");

    let toon = sb.output(&["day", "show", "2026-11-11", "--toon"]);
    let rewritten = fed(
        &sb,
        &["day", "set", "--json"],
        &String::from_utf8_lossy(&toon.stdout),
    );
    assert!(rewritten.status.success(), "{rewritten:?}");
    assert_eq!(json(&rewritten.stdout), closed_day());

    let listed = json(&sb.output(&["day", "list", "--json"]).stdout);
    assert_eq!(
        listed,
        serde_json::json!({"days": [{"day": "2026-11-11", "closed_at": "2026-11-11T22:00:00Z"}]})
    );
    assert_matches_schema(&listed, "day-list.schema.json");
}

#[test]
fn a_day_never_written_reads_as_an_open_empty_day() {
    let _guard = support::guard("a_day_never_written_reads_as_an_open_empty_day");
    let sb = Sandbox::new();
    let shown = json(&sb.output(&["day", "show", "2026-11-12", "--json"]).stdout);
    assert_eq!(
        shown,
        serde_json::json!({"day": "2026-11-12", "plan": [], "proposals": [], "decisions": [], "tasks": [], "closed_at": null})
    );
    assert_matches_schema(&shown, "day.schema.json");
}

#[test]
fn a_day_document_with_a_bad_oid_or_an_unknown_key_is_refused_and_nothing_is_kept() {
    let _guard = support::guard(
        "a_day_document_with_a_bad_oid_or_an_unknown_key_is_refused_and_nothing_is_kept",
    );
    let sb = Sandbox::new();
    let mut bad_oid = closed_day();
    bad_oid["plan"] = serde_json::json!(["abc"]);
    let mut unknown_key = closed_day();
    unknown_key["mood"] = serde_json::json!("fine");
    for doc in [bad_oid, unknown_key] {
        let out = fed(&sb, &["day", "set", "--json"], &doc.to_string());
        assert_eq!(out.status.code(), Some(1), "{out:?}");
        assert_eq!(json(&out.stderr)["error"]["kind"], "parse");
    }
    let listed = json(&sb.output(&["day", "list", "--json"]).stdout);
    assert_eq!(listed, serde_json::json!({"days": []}));
}
