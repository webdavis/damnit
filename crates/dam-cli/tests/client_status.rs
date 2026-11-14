mod support;

use dam_domain::Token;
use support::sandbox::Sandbox;
use support::token_table::{Grant, TokenTable};

#[test]
fn a_status_warning_is_in_the_document_a_client_parses() {
    let _guard = support::guard("a_status_warning_is_in_the_document_a_client_parses");
    let sb = Sandbox::new();
    let value = Token::generate(&mut |b: &mut [u8]| b.fill(1)).expose();
    sb.add_token(
        "tool",
        TokenTable::valued(&value).grant(Grant::allowing(&["task:fly"])),
    );
    let answer = support::status_json(&sb, &["--json"]);
    let warnings = answer["warnings"].as_array().expect("warnings is a list");
    assert_eq!(warnings.len(), 1, "{answer}");
    let said = warnings[0].as_str().unwrap();
    assert!(said.contains("task:fly"), "{said}");
    assert!(
        !said.contains(&value),
        "the warning carries the token: {said}"
    );
}
