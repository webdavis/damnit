use dam_application::{PullReport, RemoteName};

use super::pull_view::{PullRow, Pulled};
use crate::view::{Human, View};

fn pulled(remote: &str, counts: [usize; 5]) -> PullReport {
    let [created, updated, unchanged, conflicts, removed_upstream] = counts;
    PullReport {
        remote: RemoteName(remote.into()),
        created,
        updated,
        conflicts,
        removed_upstream,
        unchanged,
        creates_matched: 0,
        values_kept: 0,
        deletes_confirmed: 0,
        task_pulled: None,
        unseen_pulled: false,
    }
}

fn text(reports: &[PullReport]) -> Vec<String> {
    let shown = Pulled {
        remotes: reports.iter().map(PullRow::new).collect(),
    };
    let Human::Styled { plain, colored } = shown.human() else {
        panic!("expected the doctor style")
    };
    assert_ne!(plain, colored);
    plain
}

const HEADING: &str = "pull ── what each remote sent";

#[test]
fn a_remote_with_new_and_updated_tasks_names_only_those_counts() {
    assert_eq!(
        text(&[pulled("todoist", [3, 2, 0, 0, 0])]),
        [HEADING, "  ✓ todoist: 3 new, 2 updated"]
    );
}

#[test]
fn a_remote_that_sent_nothing_says_so_in_words() {
    assert_eq!(
        text(&[pulled("todoist", [0, 0, 0, 0, 0])]),
        [HEADING, "  ✓ todoist: nothing new"]
    );
}

#[test]
fn a_remote_that_sent_nothing_still_counts_what_was_unchanged() {
    assert_eq!(
        text(&[pulled("todoist", [0, 0, 2639, 0, 0])]),
        [HEADING, "  ✓ todoist: nothing new, 2639 unchanged"]
    );
}

#[test]
fn a_remote_with_conflicts_warns_and_names_the_command_to_settle_them() {
    assert_eq!(
        text(&[pulled("todoist", [1, 0, 4, 2, 0])]),
        [
            HEADING,
            "  ⚠ todoist: 1 new, 4 unchanged, 2 conflicts",
            "    → see dam status, settle each with dam resolve",
        ]
    );
}

#[test]
fn a_remote_that_removed_tasks_warns_and_names_the_command_to_drop_them() {
    assert_eq!(
        text(&[pulled("todoist", [0, 0, 0, 0, 1])]),
        [
            HEADING,
            "  ⚠ todoist: 1 removed upstream",
            "    → see dam status, drop each with dam rm",
        ]
    );
}

#[test]
fn several_remotes_print_one_row_each() {
    assert_eq!(
        text(&[
            pulled("todoist", [1, 0, 0, 0, 0]),
            pulled("gcal", [0, 0, 0, 0, 0]),
        ]),
        [HEADING, "  ✓ todoist: 1 new", "  ✓ gcal: nothing new"]
    );
}

#[test]
fn no_remotes_says_so_under_the_heading() {
    assert_eq!(text(&[]), [HEADING, "  · no remotes to pull from"]);
}
