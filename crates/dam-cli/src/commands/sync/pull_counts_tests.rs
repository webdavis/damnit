use dam_application::{PullReport, RemoteName};

use super::pull_view::{PullRow, Pulled};
use crate::view::report;

const PULL_SCHEMA: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/schemas/pull.schema.json"
));

fn pulled_taking(matched: usize, kept: usize, confirmed: usize) -> crate::output::Report {
    let pulled = PullReport {
        remote: RemoteName("fake".into()),
        created: 0,
        updated: 0,
        conflicts: 0,
        removed_upstream: 0,
        unchanged: 2,
        creates_matched: matched,
        values_kept: kept,
        deletes_confirmed: confirmed,
        task_pulled: None,
        unseen_pulled: false,
    };
    report(&Pulled {
        remotes: vec![PullRow::new(&pulled)],
    })
    .unwrap()
}

#[test]
fn a_pull_prints_what_it_took_from_the_record_of_attempts() {
    assert_eq!(
        pulled_taking(1, 1, 1).human,
        "pull ── what each remote sent\n  ✓ fake: nothing new, 2 unchanged\n    \
         · 1 create matched\n    · 1 value kept\n    · 1 delete confirmed"
    );
}

#[test]
fn a_pull_answers_what_it_took_from_the_record_of_attempts() {
    let data = pulled_taking(1, 2, 3).data;
    let schema: serde_json::Value = serde_json::from_str(PULL_SCHEMA).unwrap();
    assert!(jsonschema::is_valid(&schema, &data), "{data}");
    let row = &data["remotes"][0];
    let counts = ["creates_matched", "values_kept", "deletes_confirmed"].map(|k| &row[k]);
    assert_eq!(counts, [1, 2, 3], "{data}");
}

#[test]
fn a_pull_that_took_nothing_from_the_record_prints_only_its_row() {
    assert_eq!(pulled_taking(0, 0, 0).human.lines().count(), 2);
}
