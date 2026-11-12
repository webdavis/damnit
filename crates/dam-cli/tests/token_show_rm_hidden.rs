mod support;

use support::hidden::Hidden;

#[test]
fn a_token_showing_a_hidden_object_hears_what_a_missing_one_gets() {
    let _guard = support::guard("a_token_showing_a_hidden_object_hears_what_a_missing_one_gets");
    let h = Hidden::new();
    h.each_hidden_by_oid_and_prefix(|hidden, named| {
        let out = h.answers_alike_once_removed(hidden, &["show", named, "--no-pull", "--json"]);
        assert_eq!(out.status.code(), Some(4), "{named}");
    });
}

#[test]
fn a_token_removing_a_hidden_object_hears_what_a_missing_one_gets() {
    let _guard = support::guard("a_token_removing_a_hidden_object_hears_what_a_missing_one_gets");
    let h = Hidden::new();
    h.each_hidden_by_oid_and_prefix(|hidden, named| {
        let out = h.answers_alike_once_removed(hidden, &["rm", named, "--json"]);
        assert_eq!(out.status.code(), Some(4), "{named}");
    });
}
