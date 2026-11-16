use crate::support;

use std::path::PathBuf;
use std::process::Output;

use support::fixture::{self, OAT_MILK};
use support::golden::normalize;
use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;

const EXIT_USAGE: i32 = 2;
const EXIT_REFUSED: i32 = 4;

fn assert_fixture(name: &str, text: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../nvim/tests/fixtures")
        .join(name);
    let scrubbed = normalize(text, &fixture::seeded_oids());
    if std::env::var_os("DAM_WRITE_NVIM_FIXTURES").is_some() {
        std::fs::write(&path, &scrubbed).unwrap();
        return;
    }
    let kept = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "{}: {e}; DAM_WRITE_NVIM_FIXTURES=1 writes it",
            path.display()
        )
    });
    assert_eq!(
        scrubbed, kept,
        "dam's answer drifted from the plugin's fixture {name}; DAM_WRITE_NVIM_FIXTURES=1 rewrites it"
    );
}

fn answer(sb: &Sandbox, args: &[&str], schema: &str) -> String {
    let out = sb.output(args);
    assert!(out.status.success(), "{args:?}: {out:?}");
    let text = String::from_utf8(out.stdout).unwrap();
    assert_matches_schema(&serde_json::from_str(&text).unwrap(), schema);
    text
}

fn error_document(out: Output, code: i32) -> String {
    assert_eq!(out.status.code(), Some(code), "{out:?}");
    let text = String::from_utf8(out.stderr).unwrap();
    assert_matches_schema(&serde_json::from_str(&text).unwrap(), "error.schema.json");
    text
}

#[test]
fn the_version_banner_is_the_one_the_plugin_reads() {
    let _guard = support::guard("the_version_banner_is_the_one_the_plugin_reads");
    let out = Sandbox::new().output(&["--version"]);
    assert!(out.status.success(), "{out:?}");
    assert_fixture("version.txt", &String::from_utf8(out.stdout).unwrap());
}

#[test]
fn the_list_fixture_is_what_dam_list_answers() {
    let _guard = support::guard("the_list_fixture_is_what_dam_list_answers");
    let sb = fixture::seeded();
    sb.new_object(&["pack the bag", "--parent", &fixture::oid(OAT_MILK)]);
    let text = answer(&sb, &["list", "-o", "json"], "list.schema.json");
    assert_fixture("list.json", &text);
}

#[test]
fn the_error_fixtures_are_what_dam_prints_when_it_says_no() {
    let _guard = support::guard("the_error_fixtures_are_what_dam_prints_when_it_says_no");
    let sb = Sandbox::new();
    let selection = sb.output(&["list", "--priority", "urgent", "-o", "json"]);
    assert_fixture(
        "error-usage-selection.json",
        &error_document(selection, EXIT_USAGE),
    );
    let refused = sb.output(&["show", "0000000", "-o", "json"]);
    assert_fixture("error-refused.json", &error_document(refused, EXIT_REFUSED));
}
