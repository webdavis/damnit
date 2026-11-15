use super::{FAKE_AND_WORK, FAKE_ONLY, FAKE_SILENT, FAKE_TOKEN, Files, Helpers, json, rows_of};
use crate::args::CheckName;

#[test]
fn a_helper_that_answers_is_well_and_is_given_no_credential() {
    let files = Files::new(FAKE_ONLY, FAKE_TOKEN);
    let helpers = Helpers::default();
    let report = files.report(&helpers, &[CheckName::Helpers]);
    assert_eq!(
        rows_of(&report),
        [
            "  ✓ helpers: 1 remote: 1 ok",
            "    ✓ fake: dam-remote-fake answered",
        ]
    );
    assert_eq!(*helpers.credentials_given.borrow(), [0]);
}

#[test]
fn a_silent_helper_is_an_issue_naming_its_deadline() {
    let files = Files::new(FAKE_SILENT, FAKE_TOKEN);
    let report = files.report(&Helpers::default(), &[CheckName::Helpers]);
    let rows = rows_of(&report);
    assert!(
        rows[1].starts_with("    ✗ fake: helper fake gave no answer within 1s"),
        "{rows:?}"
    );
    assert_eq!(json(&report)["checks"][0]["result"], "issue");
}

#[test]
fn each_remote_is_a_row_named_by_the_remote_under_its_check() {
    let files = Files::new(FAKE_AND_WORK, FAKE_TOKEN);
    let report = files.report(&Helpers::default(), &[CheckName::Helpers]);
    assert_eq!(
        rows_of(&report),
        [
            "  ✗ helpers: 2 remotes: 1 issue, 1 ok",
            "    ✓ fake: dam-remote-fake answered",
            "    ✗ work: dam-remote-nosuch is not installed",
            "      → install it, then run dam doctor",
        ]
    );
    let document = json(&report);
    assert_eq!(
        document["checks"],
        serde_json::json!([{
            "name": "helpers",
            "result": "issue",
            "detail": "2 remotes: 1 issue, 1 ok",
            "advice": null,
            "rows": [
                {"name": "fake", "result": "ok", "detail": "dam-remote-fake answered", "advice": null},
                {
                    "name": "work",
                    "result": "issue",
                    "detail": "dam-remote-nosuch is not installed",
                    "advice": "install it, then run dam doctor"
                }
            ]
        }])
    );
    assert_eq!(
        document["needs_attention"],
        serde_json::json!(["helpers for work: dam-remote-nosuch is not installed"])
    );
}
