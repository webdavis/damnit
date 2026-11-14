mod support;

use std::os::unix::fs::PermissionsExt;

use dam_domain::Token;
use support::errors::error_document;
use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;
use support::token_table::TokenTable;

fn token(fill: u8) -> String {
    Token::generate(&mut |b: &mut [u8]| b.fill(fill)).expose()
}

fn secrets_of(sb: &Sandbox) -> String {
    std::fs::read_to_string(sb.secrets_path()).unwrap()
}

fn revoked(sb: &Sandbox, name: &str) -> serde_json::Value {
    let (ok, out, err) = sb.dam(&["token", "revoke", name, "--json"]);
    assert!(ok, "{err}");
    let answer: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_matches_schema(&answer, "token-revoke.schema.json");
    answer
}

#[test]
fn revoking_removes_the_token_and_leaves_every_other_byte() {
    let _guard = support::guard("revoking_removes_the_token_and_leaves_every_other_byte");
    let sb = Sandbox::new();
    let keep = format!(
        "    -- the agents\n    keep = {{ value = \"{}\" }},   -- mine\n",
        token(2)
    );
    let gone = format!(
        "    tool = {{\n      value = \"{}\",\n\n      grant = {{ {{ allow = {{ \"task:read\" }} }} }},\n    }},\n",
        token(1)
    );
    let tail = "  },\n  -- saved views\n  remote = {  fake = { api_token = \"tok-123\" } },\n}\n";
    sb.write_secrets_lua(&format!(
        "return {{\n  schema = 1,\n  token = {{\n{keep}{gone}{tail}"
    ));
    let before = secrets_of(&sb);
    let answer = revoked(&sb, "tool");
    assert_eq!(answer["name"], "tool");
    assert_eq!(secrets_of(&sb), before.replace(&gone, ""));
    let mode = std::fs::metadata(sb.secrets_path())
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o600);
    let out = sb.output_holding(&token(1), &["ls", "--json", "--no-pull"]);
    assert_eq!(out.status.code(), Some(5), "{out:?}");
    assert_eq!(error_document(&out)["error"]["rule"], "unknown_token");
}

#[test]
fn a_token_dam_cannot_read_can_still_be_revoked() {
    let _guard = support::guard("a_token_dam_cannot_read_can_still_be_revoked");
    let sb = Sandbox::new();
    let before = secrets_of(&sb);
    sb.add_token(
        "bare",
        TokenTable::without_value().expires("2000-01-01T00:00:00Z"),
    );
    revoked(&sb, "bare");
    assert_eq!(secrets_of(&sb), before);
    let (ok, out, _) = sb.dam(&["token", "revoke", "tool"]);
    assert!(!ok);
    assert!(out.is_empty());
}

#[test]
fn revoking_a_name_no_token_holds_is_refused_and_changes_nothing() {
    let _guard = support::guard("revoking_a_name_no_token_holds_is_refused_and_changes_nothing");
    let sb = Sandbox::new();
    let before = secrets_of(&sb);
    let out = sb.output(&["token", "revoke", "tool", "--json"]);
    assert_eq!(out.status.code(), Some(4), "{out:?}");
    let document = error_document(&out);
    assert_matches_schema(&document, "error.schema.json");
    assert_eq!(document["error"]["rule"], "no_such_token");
    assert_eq!(secrets_of(&sb), before);
}

#[test]
fn text_and_toon_name_the_revoked_token() {
    let _guard = support::guard("text_and_toon_name_the_revoked_token");
    let sb = Sandbox::new();
    sb.add_token("a", TokenTable::valued(&token(1)));
    sb.add_token("b", TokenTable::valued(&token(2)));
    let (ok, out, err) = sb.dam(&["token", "revoke", "a"]);
    assert!(ok, "{err}");
    assert!(out.ends_with("revoked token a\n"), "{out}");
    let (ok, out, err) = sb.dam(&["token", "revoke", "b", "--toon"]);
    assert!(ok, "{err}");
    assert!(out.contains("name: b"), "{out}");
}
