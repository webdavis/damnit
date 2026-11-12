mod support;

use support::hidden::Hidden;
use support::tokens::refused;

#[test]
fn a_token_editing_a_hidden_object_hears_what_a_missing_one_gets() {
    let _guard = support::guard("a_token_editing_a_hidden_object_hears_what_a_missing_one_gets");
    let h = Hidden::new();
    h.each_hidden_by_oid_and_prefix(|hidden, named| {
        let out =
            h.answers_alike_once_removed(hidden, &["edit", named, "--subject", "x", "--json"]);
        assert_eq!(out.status.code(), Some(4), "{named}");
    });
}

#[test]
fn a_token_is_refused_the_editor_whatever_it_names() {
    let _guard = support::guard("a_token_is_refused_the_editor_whatever_it_names");
    let h = Hidden::new();
    let diary = h.task("diary", "home");
    let out = h.answers_alike_once_removed(&diary, &["edit", "-e", &diary, "--json"]);
    assert_eq!(refused(&out), "not_open_to_tokens");
    assert_eq!(
        refused(&h.holding(&["edit", "-e", &h.report, "--json"])),
        "not_open_to_tokens"
    );
}

#[test]
fn a_token_naming_a_hidden_object_in_an_edit_flag_hears_what_a_missing_one_gets() {
    let _guard = support::guard(
        "a_token_naming_a_hidden_object_in_an_edit_flag_hears_what_a_missing_one_gets",
    );
    let h = Hidden::new();
    for flag in ["--depends", "--undepends", "--attach"] {
        let hidden = match flag {
            "--attach" => h.event("dentist", "home"),
            _ => h.task("diary", "home"),
        };
        match flag {
            "--depends" => h.owner(&["edit", &hidden, "--depends", &h.report]),
            "--undepends" => h.owner(&["edit", &h.report, "--depends", &hidden]),
            _ => {}
        }
        let line = ["edit", &h.report, flag, &hidden, "--json"];
        let out = h.answers_alike_once_removed(&hidden, &line);
        assert_eq!(refused(&out), "no_such_object", "{flag}");
    }
}
