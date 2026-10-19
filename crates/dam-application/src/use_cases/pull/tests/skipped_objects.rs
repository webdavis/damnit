use jiff::civil::date;

use super::super::*;
use super::{config, incoming, launcher, remote};
use crate::ports::{Notice, Repositories};
use crate::remote::{PullOutcome, RejectedObject};
use crate::testing::prelude::*;
use crate::testing::{EchoCredentials, FixedClock, FixedRandom, MemoryStore};

#[test]
fn a_rejection_with_no_remote_id_is_noticed_as_skipped_by_its_reason_alone() {
    let store = MemoryStore::new();
    let l = launcher(PullOutcome {
        rejected: vec![RejectedObject {
            remote_id: String::new(),
            why: "an object with no remote id was skipped".into(),
        }],
        ..PullOutcome::default()
    });
    pull(
        Repositories::of(&store),
        &l,
        &EchoCredentials,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(1),
        &config(),
        None,
    )
    .unwrap();
    assert_eq!(
        store.notices().unwrap(),
        vec![Notice::PullSkipped {
            remote: remote(),
            object: RejectedObject {
                remote_id: String::new(),
                why: "an object with no remote id was skipped".into(),
            },
        }]
    );
}

#[test]
fn one_unconvertible_object_is_skipped_with_a_notice_and_the_rest_still_pull() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    let l = launcher(PullOutcome {
        objects: vec![incoming("from todoist", "r-good")],
        rejected: vec![RejectedObject {
            remote_id: "r-bad".into(),
            why: "path: cannot read \"a//b/\"".into(),
        }],
        removed: vec![],
        cancelled: vec![],
        sync: Some("s1".into()),
    });
    let reports = pull(
        repos,
        &l,
        &EchoCredentials,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(1),
        &config(),
        None,
    )
    .unwrap();
    assert_eq!(reports[0].created, 1);
    assert_eq!(
        store.notices().unwrap(),
        vec![Notice::PullSkipped {
            remote: remote(),
            object: RejectedObject {
                remote_id: "r-bad".into(),
                why: "path: cannot read \"a//b/\"".into(),
            },
        }]
    );
    assert!(
        store
            .oid_for_remote_id(&remote(), "r-bad")
            .unwrap()
            .is_none()
    );
}
