use crate::support;

use support::hidden::{Hidden, text};

#[test]
fn a_hidden_open_dependency_does_not_block_a_tokens_done() {
    let _guard = support::guard("a_hidden_open_dependency_does_not_block_a_tokens_done");
    let h = Hidden::new();
    let diary = h.task("diary", "home");
    h.owner(&["edit", &h.report, "--depends", &diary]);
    let out = h.holding(&["done", &h.report, "--json"]);
    let said = text(&out.stderr) + &text(&out.stdout);
    assert!(out.status.success(), "{said}");
    assert!(!said.contains(&diary[..7]), "{said}");
}
