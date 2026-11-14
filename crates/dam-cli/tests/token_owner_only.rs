mod support;

use dam_domain::Token;
use support::errors::error_document;
use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;
use support::token_table::{Grant, TokenTable};

fn token(fill: u8) -> String {
    Token::generate(&mut |b: &mut [u8]| b.fill(fill)).expose()
}

#[test]
fn any_dam_token_is_refused_by_the_token_commands_before_it_is_looked_up() {
    let _guard =
        support::guard("any_dam_token_is_refused_by_the_token_commands_before_it_is_looked_up");
    let sb = Sandbox::new();
    let held = token(1);
    sb.add_token(
        "tool",
        TokenTable::valued(&held).grant(Grant::allowing(&["config:read"])),
    );
    let secrets = sb.secrets_path();
    let before = std::fs::read_to_string(&secrets).unwrap();
    for presented in [held.as_str(), "not-a-token", ""] {
        for line in [
            vec!["token", "create", "x", "--allow", "task:read", "--json"],
            vec!["token", "list", "--json"],
            vec!["token", "revoke", "tool", "--json"],
        ] {
            let out = sb.output_holding(presented, &line);
            assert_eq!(out.status.code(), Some(4), "{line:?}: {out:?}");
            assert!(out.stdout.is_empty());
            let document = error_document(&out);
            assert_matches_schema(&document, "error.schema.json");
            assert_eq!(document["error"]["rule"], "not_open_to_tokens");
        }
    }
    assert_eq!(std::fs::read_to_string(&secrets).unwrap(), before);
}
