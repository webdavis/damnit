use super::*;

#[test]
fn the_api_never_debug_prints_its_token() {
    let api = TodoistApi::new("http://example.test", "SUPERSECRETTOKEN");
    assert!(!format!("{api:?}").contains("SUPERSECRETTOKEN"));
}

#[test]
fn new_trims_a_trailing_slash_from_the_base() {
    let api = TodoistApi::new("http://example.test/", "tok");
    assert_eq!(api.base, "http://example.test");
}
