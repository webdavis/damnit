mod support;

use support::hidden::{full_oid, text};
use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to};

fn report_moved_from_friday_to_thursday() -> (Sandbox, String) {
    let sb = Sandbox::new();
    let line = [
        "Email Tanner the report",
        "--due",
        "2026-11-13",
        "--path",
        "work",
    ];
    let short = sb.new_object(&line);
    assert!(sb.dam(&["add", "--all"]).0);
    assert!(sb.dam(&["commit", "-m", "plan"]).0);
    assert!(sb.dam(&["edit", &short, "--due", "2026-11-12"]).0);
    (sb, short)
}

#[test]
fn an_owners_change_reads_as_a_minus_line_of_current_values_and_a_plus_line_of_proposed_ones() {
    let _guard = support::guard(
        "an_owners_change_reads_as_a_minus_line_of_current_values_and_a_plus_line_of_proposed_ones",
    );
    let (sb, short) = report_moved_from_friday_to_thursday();
    let status = text(&sb.output(&["status"]).stdout);
    assert!(
        status.starts_with(&format!(
            "Not staged:\n  changed  {short}  -Email Tanner the report  due: 2026-11-13  work/  you\n                    +Email Tanner the report  due: 2026-11-12\n"
        )),
        "{status}"
    );
}

#[test]
fn diff_reads_a_change_as_the_same_two_lines() {
    let _guard = support::guard("diff_reads_a_change_as_the_same_two_lines");
    let (sb, short) = report_moved_from_friday_to_thursday();
    let diff = text(&sb.output(&["diff"]).stdout);
    assert_eq!(
        diff,
        format!(
            "changed  {short}  -Email Tanner the report  due: 2026-11-13  work/  you\n                  +Email Tanner the report  due: 2026-11-12\n"
        )
    );
}

#[test]
fn a_waiting_change_keeps_its_hash_target_sender_and_age_on_the_minus_line() {
    let _guard =
        support::guard("a_waiting_change_keeps_its_hash_target_sender_and_age_on_the_minus_line");
    let sb = Sandbox::new();
    let rent = full_oid(&sb, &sb.new_object(&["Pay rent"]));
    let held = grant_to(&sb, "tool", 1, &grant("\"task:read\", \"task:change\""));
    let line = ["edit", &rent, "--subject", "Pay the rent", "--json"];
    let sent = answer(&sb.output_holding(&held, &line), "sent.schema.json");
    let hash = &sent["hash"].as_str().unwrap()[..7];
    let status = text(&sb.output(&["status"]).stdout);
    let lines: Vec<&str> = status.lines().collect();
    let minus = lines.iter().position(|l| l.contains("-Pay rent")).unwrap();
    assert!(
        lines[minus].starts_with(&format!(
            "  changed  {hash}  {}  -Pay rent  tool  ",
            &rent[..7]
        )),
        "{status}"
    );
    assert_eq!(lines[minus + 1], format!("{}+Pay the rent", " ".repeat(29)));
}
