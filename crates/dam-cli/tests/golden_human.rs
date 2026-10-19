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

#[test]
fn an_empty_list_names_its_empty_answer_in_one_sentence() {
    let sb = support::sandbox::Sandbox::new();
    assert_eq!(sb.dam(&["ls"]).1, "no objects match\n");
    assert_eq!(
        sb.dam(&["agenda", "--no-pull"]).1,
        "no events in the window\n"
    );
    assert_eq!(sb.dam(&["log"]).1, "no commits yet\n");
    assert_eq!(sb.dam(&["diff"]).1, "no changes\n");
}

#[test]
fn no_line_of_a_list_passes_a_forty_column_terminal() {
    let sb = fixture::with_staged_and_unstaged_changes();
    for args in [
        &["ls"][..],
        &["agenda", "--from", "2026-09-25", "--to", "2026-09-26"],
        &["status"],
        &["remote", "list"],
    ] {
        let out = sb.command(args).env("COLUMNS", "40").output().unwrap();
        let text = String::from_utf8(out.stdout).unwrap();
        assert!(!text.is_empty(), "{args:?}");
        for line in text.lines().filter(|l| !l.starts_with("warning:")) {
            assert!(line.chars().count() <= 40, "{args:?}: {line}");
        }
    }
}

#[test]
fn status_prints_its_warning_whole_at_forty_columns() {
    let sb = fixture::with_staged_and_unstaged_changes();
    let out = sb
        .command(&["status"])
        .env("COLUMNS", "40")
        .output()
        .unwrap();
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(
        text.lines()
            .any(|l| l.starts_with("warning:") && l.ends_with("<name>_command or <name>_env")),
        "{text}"
    );
}

#[test]
fn a_piped_list_without_columns_prints_whole_lines() {
    let sb = support::sandbox::Sandbox::new();
    let subject = "a subject far longer than any terminal would ever show in one row of a list";
    sb.new_object(&[subject]);
    let out = sb.command(&["ls"]).env_remove("COLUMNS").output().unwrap();
    assert!(String::from_utf8(out.stdout).unwrap().contains(subject));
}
