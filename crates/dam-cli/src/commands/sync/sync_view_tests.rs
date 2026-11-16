use dam_application::{PullReport, PushReport, RemoteName, SyncReport};
use dam_domain::Oid;

use super::sync_view::Synced;
use crate::view;

fn synced(pulled: PullReport, pushed: PushReport) -> String {
    let report = SyncReport { pulled, pushed };
    view::report(&Synced::new(&report)).unwrap().human
}

fn pulled(created: usize, conflicts: usize) -> PullReport {
    PullReport {
        remote: RemoteName("todoist".into()),
        created,
        updated: 0,
        conflicts,
        removed_upstream: 0,
        unchanged: 0,
        creates_matched: 0,
        values_kept: 0,
        deletes_confirmed: 0,
        task_pulled: None,
        unseen_pulled: false,
    }
}

fn pushed(sent: usize, failed: Vec<(Oid, String)>, held: usize) -> PushReport {
    PushReport {
        remote: RemoteName("todoist".into()),
        sent,
        succeeded: sent - failed.len(),
        failed,
        skipped: 0,
        held: Some(held),
        unseen_failed: false,
    }
}

#[test]
fn a_sync_shows_the_pull_then_the_push_leaving_out_zero_counts() {
    assert_eq!(
        synced(pulled(2, 0), pushed(1, vec![], 0)),
        "sync ── what todoist sent, then what it took\n  ✓ pulled: 2 new\n  ✓ pushed: 1 sent"
    );
}

#[test]
fn a_sync_with_nothing_to_do_says_so() {
    assert_eq!(
        synced(pulled(0, 0), pushed(0, vec![], 0)),
        "sync ── what todoist sent, then what it took\n  ✓ pulled: nothing new\n  \
         ✓ pushed: nothing to send"
    );
}

#[test]
fn conflict_advice_is_printed_once_under_the_first_row_it_applies_to() {
    assert_eq!(
        synced(pulled(0, 1), pushed(1, vec![], 1)),
        "sync ── what todoist sent, then what it took\n  ⚠ pulled: 1 conflict\n    \
         → see dam status, settle each with dam resolve\n  ⚠ pushed: 1 sent, 1 held"
    );
    assert_eq!(
        synced(pulled(0, 0), pushed(1, vec![], 1)),
        "sync ── what todoist sent, then what it took\n  ✓ pulled: nothing new\n  \
         ⚠ pushed: 1 sent, 1 held\n    → see dam status, settle each with dam resolve"
    );
}

#[test]
fn a_failed_object_is_named_under_the_push() {
    let oid = Oid::generate(&mut |b: &mut [u8]| b.fill(0xab));
    let shown = synced(
        pulled(0, 0),
        pushed(2, vec![(oid.clone(), "refused".into())], 0),
    );
    assert!(
        shown.ends_with(&format!(
            "  ✗ pushed: 2 sent, 1 failed\n    ✗ {}: refused",
            oid.short()
        )),
        "{shown}"
    );
}

#[test]
fn the_json_holds_the_pull_and_the_push_counts() {
    let report = SyncReport {
        pulled: pulled(2, 0),
        pushed: pushed(1, vec![], 0),
    };
    let json = view::report(&Synced::new(&report)).unwrap().data;
    let schema: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/schemas/sync-run.schema.json"
    )))
    .unwrap();
    assert!(jsonschema::is_valid(&schema, &json), "{json}");
    assert_eq!(json["remote"], "todoist");
    assert_eq!(json["pulled"]["created"], 2);
    assert_eq!(json["pushed"]["sent"], 1);
}
