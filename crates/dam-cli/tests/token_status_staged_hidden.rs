mod support;

use support::hidden::{Hidden, text};
use support::tokens::{grant, grant_to};

const LINES: [&[&str]; 4] = [
    &["status"],
    &["status", "--full"],
    &["status", "--json"],
    &["status", "--full", "--json"],
];

fn answers(h: &Hidden, held: &str) -> Vec<(Option<i32>, String, String)> {
    LINES
        .iter()
        .map(|line| {
            let out = h.sb.output_holding(held, line);
            (out.status.code(), text(&out.stdout), text(&out.stderr))
        })
        .collect()
}

#[test]
fn status_shows_a_path_restricted_token_only_the_staged_and_unstaged_rows_it_sees() {
    let _guard = support::guard(
        "status_shows_a_path_restricted_token_only_the_staged_and_unstaged_rows_it_sees",
    );
    let h = Hidden::new();
    let held = grant_to(
        &h.sb,
        "reader",
        2,
        vec![grant(&["task:read", "stage:read", "working:read"]).paths(&["work"])],
    );
    let diary = h.task("diary", "home");
    h.owner(&["add", &diary]);
    h.owner(&["edit", &diary, "--subject", "the diary"]);
    h.owner(&["edit", &h.report, "--depends", &diary]);
    h.owner(&["add", &h.report]);
    h.owner(&["edit", &h.report, "--subject", "the report"]);

    let while_hidden = answers(&h, &held);
    for (code, stdout, _) in &while_hidden {
        assert_eq!(*code, Some(0), "{stdout}");
        assert!(stdout.contains("the report"), "{stdout}");
        assert!(!stdout.contains(&diary), "{stdout}");
        assert!(!stdout.contains(&diary[..7]), "{stdout}");
    }
    h.owner(&["reset", &diary]);
    h.owner(&["rm", &diary]);
    assert_eq!(while_hidden, answers(&h, &held));
}

#[test]
fn full_status_under_whole_stage_and_working_reads_hides_a_hidden_dependency() {
    let _guard =
        support::guard("full_status_under_whole_stage_and_working_reads_hides_a_hidden_dependency");
    let h = Hidden::new();
    let held = grant_to(
        &h.sb,
        "reader",
        2,
        vec![
            grant(&["stage:read", "working:read"]),
            grant(&["task:read"]).paths(&["work"]),
        ],
    );
    let diary = h.task("diary", "home");
    h.owner(&["add", &diary]);
    h.owner(&["commit", "-m", "diary"]);
    h.owner(&["edit", &h.report, "--depends", &diary]);
    h.owner(&["add", &h.report]);
    h.owner(&["edit", &h.report, "--subject", "the report"]);

    for (code, stdout, _) in answers(&h, &held) {
        assert_eq!(code, Some(0), "{stdout}");
        assert!(stdout.contains("the report"), "{stdout}");
        assert!(!stdout.contains(&diary[..7]), "{stdout}");
    }
}
