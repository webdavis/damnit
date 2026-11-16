use crate::nvim_fixtures::{answer, assert_fixture, error_document};
use crate::support;

use support::fixture::{self, CALL_MOM, OAT_MILK, oid, run, short};
use support::request::fed;
use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;

const EXIT_USAGE: i32 = 2;
const EXIT_REFUSED: i32 = 4;

fn fed_answer(sb: &Sandbox, args: &[&str], oids: &[&str], schema: &str) -> String {
    let input = oids.join("\n") + "\n";
    let out = fed(sb, args, &input);
    assert!(out.status.success(), "{args:?}: {out:?}");
    let text = String::from_utf8(out.stdout).unwrap();
    assert_matches_schema(&serde_json::from_str(&text).unwrap(), schema);
    text
}

fn with_rent() -> (Sandbox, String) {
    let sb = fixture::committed();
    sb.new_object(&["pack the bag", "--parent", &oid(OAT_MILK)]);
    let rent = sb.new_object(&["pay rent", "--due", "2026-09-30"]);
    (sb, rent)
}

#[test]
fn the_change_preview_fixtures_are_what_the_change_keys_read() {
    let _guard = support::guard("the_change_preview_fixtures_are_what_the_change_keys_read")
        .allow_slow("it previews every command a change key sends, one dam process each");
    let (sb, rent) = with_rent();
    let mom = oid(CALL_MOM);
    let two = [mom.as_str(), rent.as_str()];
    let done = ["done", "-", "--expect", "2", "--preview", "-o", "json"];
    let previewed = fed_answer(&sb, &done, &two, "changes.schema.json");
    assert_fixture("done-preview.json", &previewed);
    let three = [oid(OAT_MILK), mom.clone(), rent.clone()];
    let three: Vec<&str> = three.iter().map(String::as_str).collect();
    let due = [
        "edit",
        "-",
        "--due",
        "2026-11-20",
        "--expect",
        "3",
        "--preview",
        "-o",
        "json",
    ];
    assert_fixture(
        "edit-preview.json",
        &fed_answer(&sb, &due, &three, "changes.schema.json"),
    );
    let moved = ["mv", &rent, "--path", "work/", "--preview", "-o", "json"];
    assert_fixture(
        "mv-preview.json",
        &answer(&sb, &moved, "changes.schema.json"),
    );
    let removed = ["rm", "-", "--expect", "2", "--preview", "-o", "json"];
    assert_fixture(
        "rm-preview.json",
        &fed_answer(&sb, &removed, &two, "changes.schema.json"),
    );
    run(
        &sb,
        &["edit", &short(CALL_MOM), "--subject", "call mom back"],
    );
    let restore = ["restore", &mom, "--preview", "-o", "json"];
    assert_fixture(
        "restore-preview.json",
        &answer(&sb, &restore, "changes.schema.json"),
    );
}

#[test]
fn the_push_preview_fixture_is_what_status_p_confirms() {
    let _guard = support::guard("the_push_preview_fixture_is_what_status_p_confirms");
    let sb = fixture::pulled_and_pushed();
    sb.new_object(&["pay rent"]);
    run(&sb, &["add", "-A"]);
    run(&sb, &["commit", "-m", "rent"]);
    let push = ["push", "--preview", "-o", "json"];
    assert_fixture(
        "push-preview.json",
        &answer(&sb, &push, "push-preview.schema.json"),
    );
}

#[test]
fn the_change_answer_fixtures_are_what_the_change_keys_write() {
    let _guard = support::guard("the_change_answer_fixtures_are_what_the_change_keys_write")
        .allow_slow("it writes with every command a change key sends, one dam process each");
    let (sb, rent) = with_rent();
    let mom = oid(CALL_MOM);
    let one = answer(&sb, &["done", &mom, "-o", "json"], "done.schema.json");
    assert_fixture("done.json", &one);
    let three = [oid(OAT_MILK), mom.clone(), rent.clone()];
    let three: Vec<&str> = three.iter().map(String::as_str).collect();
    let priority = [
        "edit",
        "-",
        "--priority",
        "2",
        "--expect",
        "3",
        "-o",
        "json",
    ];
    assert_fixture(
        "edit-many.json",
        &fed_answer(&sb, &priority, &three, "changes.schema.json"),
    );
    let restored = answer(&sb, &["restore", &mom, "-o", "json"], "restore.schema.json");
    assert_fixture("restore.json", &restored);
}

#[test]
fn the_change_refusal_fixtures_are_what_dam_says_when_a_change_key_is_refused() {
    let _guard = support::guard(
        "the_change_refusal_fixtures_are_what_dam_says_when_a_change_key_is_refused",
    );
    let (sb, rent) = with_rent();
    let parent = sb.output(&["done", &oid(OAT_MILK), "-o", "json"]);
    assert_fixture(
        "error-done-open-children.json",
        &error_document(parent, EXIT_REFUSED),
    );
    let input = [oid(CALL_MOM), rent].join("\n") + "\n";
    let expect = fed(&sb, &["done", "-", "--expect", "3", "-o", "json"], &input);
    assert_fixture("error-expect.json", &error_document(expect, EXIT_REFUSED));
    let due = [
        "edit",
        &oid(CALL_MOM),
        "--due",
        "the twelfth of never",
        "--preview",
        "-o",
        "json",
    ];
    assert_fixture(
        "error-edit-due.json",
        &error_document(sb.output(&due), EXIT_USAGE),
    );
}
