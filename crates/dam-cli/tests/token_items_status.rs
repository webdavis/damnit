mod support;

use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;
use support::tokens::{answer, grant, grant_to};

fn status_as(sb: &Sandbox, held: Option<&str>) -> serde_json::Value {
    let out = match held {
        Some(held) => sb.output_holding(held, &["status", "--json"]),
        None => sb.output(&["status", "--json"]),
    };
    let status = answer(&out, "status.schema.json");
    assert_matches_schema(&status, "status.schema.json");
    status
}

fn dentist_from_vpt() -> (Sandbox, String, serde_json::Value) {
    let sb = Sandbox::new();
    let held = grant_to(&sb, "vpt", 1, &grant("\"task:create\""));
    let line = ["new", "Call the dentist", "--path", "inbox", "--json"];
    let sent = answer(&sb.output_holding(&held, &line), "sent.schema.json");
    (sb, held, sent)
}

#[test]
fn the_owner_sees_a_tokens_item_beside_their_own_changes() {
    let _guard = support::guard("the_owner_sees_a_tokens_item_beside_their_own_changes");
    let (sb, _, sent) = dentist_from_vpt();
    sb.new_object(&["buy milk"]);
    let status = status_as(&sb, None);
    assert_eq!(status["unstaged"][0]["subject"], "buy milk");
    assert_eq!(status["unstaged"][0]["from"], serde_json::Value::Null);
    let waiting = &status["waiting"][0];
    assert_eq!(waiting["hash"], sent["hash"]);
    assert_eq!(waiting["item"], "new");
    assert_eq!(waiting["from"], "vpt");
    assert_eq!(waiting["key_id"], sent["key_id"]);
    assert_eq!(waiting["channel"], "local");
    assert_eq!(waiting["created"], sent["created"]);
    let (ok, text, _) = sb.dam(&["status"]);
    assert!(ok);
    let short = &sent["hash"].as_str().unwrap()[..7];
    let not_staged: Vec<&str> = text.lines().skip_while(|l| *l != "Not staged:").collect();
    assert!(not_staged[1].ends_with("buy milk  you"), "{text}");
    assert!(
        not_staged[2].starts_with(&format!(
            "  new      {short}  Call the dentist  inbox/  vpt  "
        )),
        "{text}"
    );
}

#[test]
fn a_token_sees_its_own_items_and_another_senders_only_with_pending_read() {
    let _guard =
        support::guard("a_token_sees_its_own_items_and_another_senders_only_with_pending_read");
    let (sb, vpt, sent) = dentist_from_vpt();
    sb.new_object(&["buy milk"]);
    let own = status_as(&sb, Some(&vpt));
    assert_eq!(own["waiting"][0]["hash"], sent["hash"]);
    assert_eq!(own["unstaged"], serde_json::json!([]));
    let codex = grant_to(
        &sb,
        "codex",
        2,
        "[[token.codex.grant]]\nallow = [\"task:create\"]\n",
    );
    assert_eq!(
        status_as(&sb, Some(&codex))["waiting"],
        serde_json::json!([])
    );
    let claude = grant_to(
        &sb,
        "claude",
        3,
        "[[token.claude.grant]]\nallow = [\"pending:read\"]\n",
    );
    assert_eq!(
        status_as(&sb, Some(&claude))["waiting"][0]["hash"],
        sent["hash"]
    );
}
