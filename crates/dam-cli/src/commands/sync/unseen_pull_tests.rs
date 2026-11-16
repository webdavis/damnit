use dam_application::{PullReport, RemoteName};

use super::pull_view::{PullRow, Pulled};
use crate::view::report;

const PULL_SCHEMA: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/schemas/pull.schema.json"
));

const UNSEEN: &str = "some changes you cannot see came in too";

fn pulled(updated: usize, unseen_pulled: bool) -> crate::output::Report {
    let pulled = PullReport {
        remote: RemoteName("fake".into()),
        created: 0,
        updated,
        conflicts: 0,
        removed_upstream: 0,
        unchanged: 0,
        creates_matched: 0,
        values_kept: 0,
        deletes_confirmed: 0,
        task_pulled: None,
        unseen_pulled,
    };
    report(&Pulled {
        remotes: vec![PullRow::new(&pulled)],
    })
    .unwrap()
}

#[test]
fn a_token_is_told_in_one_sentence_that_changes_it_cannot_see_came_in() {
    let shown = pulled(1, true);
    assert_eq!(
        shown.human,
        format!("pull ── what each remote sent\n  ✓ fake: 1 updated\n    · {UNSEEN}")
    );
    let schema: serde_json::Value = serde_json::from_str(PULL_SCHEMA).unwrap();
    assert!(jsonschema::is_valid(&schema, &shown.data), "{}", shown.data);
    assert_eq!(shown.data["remotes"][0]["unseen_pulled"], true);
}

#[test]
fn a_token_that_can_see_nothing_new_is_not_told_nothing_came() {
    let shown = pulled(0, true);
    assert!(!shown.human.contains("nothing new"), "{}", shown.human);
}

#[test]
fn an_answer_with_nothing_unseen_has_no_unseen_key() {
    let shown = pulled(1, false);
    assert!(shown.data["remotes"][0].get("unseen_pulled").is_none());
}
