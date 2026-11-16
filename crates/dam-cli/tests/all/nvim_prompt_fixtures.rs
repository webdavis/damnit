use crate::nvim_fixtures::{answer, assert_fixture, error_document};
use crate::support;

use support::fixture;

const EXIT_USAGE: i32 = 2;

#[test]
fn the_new_preview_fixtures_are_what_quick_add_reads_while_you_type() {
    let _guard = support::guard("the_new_preview_fixtures_are_what_quick_add_reads_while_you_type");
    let sb = fixture::seeded();
    let flags = [
        "--due",
        "2026-11-20",
        "--labels",
        "errand",
        "--path",
        "home/",
    ];
    let previewed = [
        &["new", "buy oat milk"],
        &flags[..],
        &["--preview", "-o", "json"],
    ]
    .concat();
    assert_fixture(
        "new-preview.json",
        &answer(&sb, &previewed, "changes.schema.json"),
    );
    let refused = sb.output(&[
        "new",
        "buy oat milk",
        "--due",
        "the twelfth of never",
        "--preview",
        "-o",
        "json",
    ]);
    assert_fixture(
        "error-new-preview-due.json",
        &error_document(refused, EXIT_USAGE),
    );
    let made = [&["new", "buy oat milk"], &flags[..], &["-o", "json"]].concat();
    assert_fixture("new.json", &answer(&sb, &made, "new.schema.json"));
}

#[test]
fn the_vocabulary_fixtures_are_what_dam_lists_for_completion() {
    let _guard = support::guard("the_vocabulary_fixtures_are_what_dam_lists_for_completion");
    let sb = fixture::seeded();
    let categories = ["category", "list", "-o", "json"];
    assert_fixture(
        "category-list.json",
        &answer(&sb, &categories, "category-list.schema.json"),
    );
    let remotes = ["remote", "list", "-o", "json"];
    assert_fixture(
        "remote-list.json",
        &answer(&sb, &remotes, "remote-list.schema.json"),
    );
    let unknown = sb.output(&["nonesuch", "list", "-o", "json"]);
    assert_fixture(
        "error-unknown-subcommand.json",
        &error_document(unknown, EXIT_USAGE),
    );
}
