mod support;

use support::fixture::{self, OAT_MILK};
use support::golden::assert_golden_output;

#[test]
fn ls_prints_the_golden_document() {
    assert_golden_output(&fixture::committed(), &["--json", "ls"], "ls.json");
}

#[test]
fn show_prints_the_golden_document() {
    let sb = fixture::committed();
    assert_golden_output(
        &sb,
        &["--json", "show", &fixture::short(OAT_MILK)],
        "show.json",
    );
}

#[test]
fn status_prints_the_golden_document() {
    let sb = fixture::with_staged_and_unstaged_changes();
    assert_golden_output(&sb, &["--json", "status"], "status.json");
}

#[test]
fn status_full_prints_the_golden_document() {
    let sb = fixture::with_staged_and_unstaged_changes();
    assert_golden_output(&sb, &["--json", "status", "--full"], "status_full.json");
}

#[test]
fn diff_prints_the_golden_document() {
    let sb = fixture::with_staged_and_unstaged_changes();
    assert_golden_output(&sb, &["--json", "diff"], "diff.json");
}

#[test]
fn log_prints_the_golden_document() {
    assert_golden_output(&fixture::with_two_commits(), &["--json", "log"], "log.json");
}

#[test]
fn agenda_prints_the_golden_document() {
    let args = [
        "--json",
        "agenda",
        "--from",
        "2026-09-25",
        "--to",
        "2026-09-26",
    ];
    assert_golden_output(&fixture::committed(), &args, "agenda.json");
}

#[test]
fn category_list_prints_the_golden_document() {
    let sb = fixture::seeded();
    assert_golden_output(&sb, &["--json", "category", "list"], "category_list.json");
}

#[test]
fn filter_list_prints_the_golden_document() {
    let sb = fixture::seeded();
    assert_golden_output(&sb, &["--json", "filter", "list"], "filter_list.json");
}

#[test]
fn remote_list_prints_the_golden_document() {
    let sb = fixture::pulled_and_pushed();
    assert_golden_output(&sb, &["--json", "remote", "list"], "remote_list.json");
}
