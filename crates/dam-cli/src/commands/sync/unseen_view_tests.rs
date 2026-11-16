use dam_application::{PushReport, RemoteName};

use super::view::{PushRow, Pushed};
use crate::style::Paint;
use crate::view::{Human, View};

const UNSEEN: &str =
    "some changes you cannot see did not go through; ask the owner to run dam status";

fn pushed(held: Option<usize>, unseen_failed: bool) -> PushReport {
    PushReport {
        remote: RemoteName("fake".into()),
        sent: 0,
        succeeded: 0,
        failed: vec![],
        skipped: 0,
        held,
        unseen_failed,
    }
}

fn text(report: &PushReport) -> Vec<String> {
    let shown = Pushed {
        remotes: vec![PushRow::new(report)],
    };
    let Human::Lines(lines) = shown.human() else {
        panic!("expected plain lines")
    };
    lines
}

#[test]
fn a_token_is_told_in_one_sentence_that_something_it_cannot_see_did_not_go_through() {
    let report = pushed(None, true);
    assert_eq!(
        text(&report),
        [format!(
            "fake: 0 sent, 0 ok, 0 failed, 0 skipped\n  {UNSEEN}"
        )]
    );
    let row = serde_json::to_value(PushRow::new(&report)).unwrap();
    assert_eq!(row["unseen_failed"], serde_json::Value::Bool(true));
}

#[test]
fn the_sentence_shows_as_a_failure_in_the_sync_lines() {
    let report = pushed(Some(0), true);
    let lines = PushRow::new(&report).lines(Paint::Plain, "pushed");
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert!(lines[0].starts_with("  ✗ pushed"), "{lines:?}");
    assert!(lines[1].ends_with(UNSEEN), "{lines:?}");
}

#[test]
fn an_answer_with_nothing_unseen_has_no_unseen_key() {
    let row = serde_json::to_value(PushRow::new(&pushed(Some(0), false))).unwrap();
    assert!(row.get("unseen_failed").is_none(), "{row}");
}
