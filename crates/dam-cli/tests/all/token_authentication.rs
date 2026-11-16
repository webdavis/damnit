use crate::support;

use dam_domain::Token;
use support::errors::error_document;
use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;
use support::token_table::TokenTable;

fn token(fill: u8) -> String {
    Token::generate(&mut |b: &mut [u8]| b.fill(fill)).expose()
}

fn sandbox_with(tokens: Vec<(&str, TokenTable)>) -> Sandbox {
    let sb = Sandbox::new();
    for (name, table) in tokens {
        sb.add_token(name, table);
    }
    sb
}

fn refused(sb: &Sandbox, presented: &str, args: &[&str]) -> serde_json::Value {
    let out = sb.output_holding(presented, args);
    assert_eq!(out.status.code(), Some(5), "{out:?}");
    assert!(out.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        presented.is_empty() || !stderr.contains(presented),
        "the token reached stderr"
    );
    let document = error_document(&out);
    assert_matches_schema(&document, "error.schema.json");
    assert_eq!(document["error"]["kind"], "refused");
    document
}

fn rule(document: &serde_json::Value) -> &str {
    document["error"]["rule"].as_str().unwrap()
}

#[test]
fn a_well_formed_token_the_config_does_not_list_is_unknown() {
    let _guard = support::guard("a_well_formed_token_the_config_does_not_list_is_unknown");
    let sb = sandbox_with(vec![("tool", TokenTable::valued(&token(1)))]);
    let document = refused(&sb, &token(2), &["list", "-o", "json"]);
    assert_eq!(rule(&document), "unknown_token");
}

#[test]
fn reading_the_config_alone_still_checks_the_token() {
    let _guard = support::guard("reading_the_config_alone_still_checks_the_token");
    let sb = sandbox_with(vec![]);
    let document = refused(&sb, &token(2), &["category", "list", "-o", "json"]);
    assert_eq!(rule(&document), "unknown_token");
}
