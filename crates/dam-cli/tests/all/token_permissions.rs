use crate::support;

use dam_domain::Token;
use support::errors::error_document;
use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;
use support::token_table::{Grant, TokenTable};

fn token(fill: u8) -> String {
    Token::generate(&mut |b: &mut [u8]| b.fill(fill)).expose()
}

fn rule_of(out: &std::process::Output) -> String {
    assert_eq!(out.status.code(), Some(4), "{out:?}");
    assert_matches_schema(&error_document(out), "error.schema.json");
    error_document(out)["error"]["rule"]
        .as_str()
        .unwrap()
        .to_string()
}

#[test]
fn a_token_reaches_only_the_remotes_its_grant_names() {
    let _guard = support::guard("a_token_reaches_only_the_remotes_its_grant_names");
    let sb = Sandbox::new();
    let held = token(1);
    sb.add_token(
        "tool",
        TokenTable::valued(&held)
            .grant(Grant::allowing(&["remote:pull", "remote:push"]).remotes(&["elsewhere"])),
    );
    for verb in ["pull", "push"] {
        for named in ["fake", "nope"] {
            let out = sb.output_holding(&held, &[verb, named, "--json"]);
            assert_eq!(rule_of(&out), "not_permitted", "{verb} {named}");
        }
        let out = sb.output_holding(&held, &[verb, "--json"]);
        assert!(
            out.status.success(),
            "{verb} of every remote narrows to none: {out:?}"
        );
    }
    assert!(sb.pushes().is_empty());
}
