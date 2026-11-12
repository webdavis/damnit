mod support;

use support::hidden::{Hidden, text};

#[test]
fn a_token_completing_a_hidden_object_hears_what_a_missing_one_gets() {
    let _guard = support::guard("a_token_completing_a_hidden_object_hears_what_a_missing_one_gets");
    let h = Hidden::new();
    h.each_hidden_by_oid_and_prefix(|hidden, named| {
        let out = h.answers_alike_once_removed(hidden, &["done", named, "--json"]);
        assert_eq!(out.status.code(), Some(4), "{named}");
    });
}

#[test]
fn a_hidden_open_dependency_does_not_block_a_tokens_done() {
    let _guard = support::guard("a_hidden_open_dependency_does_not_block_a_tokens_done");
    let h = Hidden::new();
    let diary = h.task("diary", "home");
    h.owner(&["edit", &h.report, "--depends", &diary]);
    let out = h.holding(&["done", &h.report, "--json"]);
    let said = text(&out.stderr) + &text(&out.stdout);
    assert!(out.status.success(), "{said}");
    assert!(!said.contains(&diary[..4]), "{said}");
}
