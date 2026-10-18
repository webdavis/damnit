mod support;

use support::sandbox::Sandbox;

#[test]
fn a_status_warning_is_in_the_document_a_client_parses() {
    let _guard = support::guard("a_status_warning_is_in_the_document_a_client_parses");
    let sb = Sandbox::new();
    let answer = support::status_json(&sb, &["--json"]);
    let warnings = answer["warnings"].as_array().expect("warnings is a list");
    assert_eq!(warnings.len(), 1, "{answer}");
    let said = warnings[0].as_str().unwrap();
    assert!(said.contains("fake.api_token"), "{said}");
    assert!(
        !said.contains("tok-123"),
        "the advisory carries the secret: {said}"
    );
}
