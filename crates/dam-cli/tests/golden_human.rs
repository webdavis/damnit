mod support;

use support::fixture;
use support::golden::assert_golden_output;

#[test]
fn ls_prints_the_golden_lines() {
    assert_golden_output(&fixture::committed(), &["ls"], "ls.txt");
}

#[test]
fn agenda_prints_the_golden_lines() {
    let args = ["agenda", "--from", "2026-09-25", "--to", "2026-09-26"];
    assert_golden_output(&fixture::committed(), &args, "agenda.txt");
}

#[test]
fn status_prints_the_golden_lines() {
    let sb = fixture::with_staged_and_unstaged_changes();
    assert_golden_output(&sb, &["status"], "status.txt");
}

#[test]
fn log_prints_the_golden_lines() {
    assert_golden_output(&fixture::with_two_commits(), &["log"], "log.txt");
}

#[test]
fn remote_list_prints_the_golden_lines() {
    let sb = fixture::pulled_and_pushed();
    assert_golden_output(&sb, &["remote", "list"], "remote_list.txt");
}

#[test]
fn category_list_prints_the_golden_lines() {
    assert_golden_output(
        &fixture::seeded(),
        &["category", "list"],
        "category_list.txt",
    );
}

#[test]
fn filter_list_prints_the_golden_lines() {
    assert_golden_output(&fixture::seeded(), &["filter", "list"], "filter_list.txt");
}
