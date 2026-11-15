use dam_adapters::SqliteStore;
use dam_application::testing::{event, oid, task};
use dam_application::{
    AcceptedKindsRepository, Attempt, AttemptRepository, CommitRepository, MutationOp, RemoteName,
};
use dam_domain::{Authored, Change, CommitId, CommitRecord, Kind, Timestamp};

use super::{FAKE_ONLY, FAKE_TOKEN, Files, Helpers, rows_of};
use crate::args::CheckName;

const PUSH: &str = "      → run dam push";

fn store_owing(files: &Files, retries: &[u8], unanswered: &[u8]) {
    let store = SqliteStore::open(&files.store_path()).unwrap();
    let fake = RemoteName("fake".into());
    let retries: Vec<_> = retries.iter().map(|n| oid(*n)).collect();
    store.set_push_retries(&fake, &retries).unwrap();
    for n in unanswered {
        let attempt = Attempt {
            key: format!("key-{n}"),
            op: MutationOp::Create,
            container_remote_id: None,
            declared: Default::default(),
        };
        store.set_attempts(&fake, &oid(*n), &[attempt]).unwrap();
    }
}

fn pushes(files: &Files) -> Vec<String> {
    let helpers = Helpers::default();
    let rows = rows_of(&files.report(&helpers, &[CheckName::Pushes]));
    assert!(helpers.credentials_given.borrow().is_empty());
    rows
}

#[test]
fn changes_held_for_the_next_push_are_a_warning() {
    let files = Files::new(FAKE_ONLY, FAKE_TOKEN);
    store_owing(&files, &[1, 2], &[]);
    assert_eq!(
        pushes(&files),
        [
            "  ⚠ pushes: 1 remote: 1 warning",
            "    ⚠ fake: 2 changes still owed",
            PUSH
        ]
    );
}

#[test]
fn a_change_sent_without_an_answer_is_a_warning_beside_what_is_owed() {
    let files = Files::new(FAKE_ONLY, FAKE_TOKEN);
    store_owing(&files, &[1], &[2]);
    assert_eq!(
        pushes(&files),
        [
            "  ⚠ pushes: 1 remote: 1 warning",
            "    ⚠ fake: 1 change still owed, 1 change sent without an answer",
            PUSH
        ]
    );
}

#[test]
fn a_store_that_owes_nothing_is_well() {
    let files = Files::new(FAKE_ONLY, FAKE_TOKEN);
    store_owing(&files, &[], &[]);
    assert_eq!(
        pushes(&files),
        ["  ✓ pushes: 1 remote: 1 ok", "    ✓ fake: nothing owed"]
    );
}

#[test]
fn a_change_of_a_kind_the_remote_does_not_take_is_not_owed() {
    let files = Files::new(FAKE_ONLY, FAKE_TOKEN);
    let store = SqliteStore::open(&files.store_path()).unwrap();
    let fake = RemoteName("fake".into());
    store.set_accepted_kinds(&fake, &[Kind::Task]).unwrap();
    let created = |after| Authored::by_owner(Change::Create { after });
    let record = CommitRecord {
        id: CommitId::generate(&mut |b: &mut [u8]| b.fill(1)),
        message: "both kinds".into(),
        pulled: false,
        at: Timestamp::UNIX_EPOCH,
        changes: vec![created(task(1)), created(event(2))],
    };
    store.commit(&record).unwrap();
    drop(store);
    assert_eq!(
        pushes(&files),
        [
            "  ⚠ pushes: 1 remote: 1 warning",
            "    ⚠ fake: 1 change still owed",
            PUSH
        ]
    );
}
