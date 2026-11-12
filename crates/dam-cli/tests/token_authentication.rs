mod support;

use dam_domain::Token;
use support::errors::error_document;
use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;

fn token(fill: u8) -> String {
    Token::generate(&mut |b: &mut [u8]| b.fill(fill)).expose()
}

fn sandbox_with(token_tables: &str) -> Sandbox {
    let sb = Sandbox::new();
    sb.add_config(token_tables);
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
fn without_dam_token_the_caller_is_the_owner() {
    let _guard = support::guard("without_dam_token_the_caller_is_the_owner");
    let sb = sandbox_with(&format!("[token.vpt]\nvalue = \"{}\"\n", token(1)));
    let (ok, _, err) = sb.dam(&["ls", "--json", "--no-pull"]);
    assert!(ok, "{err}");
}

#[test]
fn a_token_that_is_not_well_formed_is_refused_as_malformed() {
    let _guard = support::guard("a_token_that_is_not_well_formed_is_refused_as_malformed");
    let sb = sandbox_with("");
    for presented in [
        "",
        "hunter2-secret",
        "damk_v1_tooshort",
        "damk_s3cr3tValue_abcdef",
    ] {
        let document = refused(&sb, presented, &["ls", "--json", "--no-pull"]);
        assert_eq!(rule(&document), "malformed_token");
    }
    let document = refused(&sb, "damk_v7_abcdef", &["ls", "--json", "--no-pull"]);
    assert_eq!(rule(&document), "unknown_token_version");
}

#[test]
fn a_well_formed_token_the_config_does_not_list_is_unknown() {
    let _guard = support::guard("a_well_formed_token_the_config_does_not_list_is_unknown");
    let sb = sandbox_with(&format!("[token.vpt]\nvalue = \"{}\"\n", token(1)));
    let document = refused(&sb, &token(2), &["ls", "--json", "--no-pull"]);
    assert_eq!(rule(&document), "unknown_token");
}

#[test]
fn disabled_expired_early_and_wrong_channel_tokens_each_name_their_rule() {
    let _guard =
        support::guard("disabled_expired_early_and_wrong_channel_tokens_each_name_their_rule");
    let sb = sandbox_with(&format!(
        "[token.a]\nvalue = \"{}\"\nweekdays = [\"mon\"]\n\
         [token.b]\nvalue = \"{}\"\nexpires = 2000-01-01T00:00:00Z\n\
         [token.c]\nvalue = \"{}\"\nnot_before = 2999-01-01T00:00:00Z\n\
         [token.d]\nvalue = \"{}\"\nvia = [\"mcp\"]\n",
        token(1),
        token(2),
        token(3),
        token(4)
    ));
    let cases = [
        (token(1), "disabled_token"),
        (token(2), "expired_token"),
        (token(3), "token_not_yet_valid"),
        (token(4), "channel_not_allowed"),
    ];
    for (presented, expected) in cases {
        let document = refused(&sb, &presented, &["ls", "--json", "--no-pull"]);
        assert_eq!(rule(&document), expected);
    }
    let disabled = refused(&sb, &token(1), &["ls", "--json", "--no-pull"]);
    assert!(
        disabled["error"]["message"]
            .as_str()
            .unwrap()
            .contains("weekdays")
    );
}

#[test]
fn reading_the_config_alone_still_checks_the_token() {
    let _guard = support::guard("reading_the_config_alone_still_checks_the_token");
    let sb = sandbox_with("");
    let document = refused(&sb, &token(2), &["category", "list", "--json"]);
    assert_eq!(rule(&document), "unknown_token");
}
