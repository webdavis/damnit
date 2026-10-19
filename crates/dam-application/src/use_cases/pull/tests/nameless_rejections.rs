use jiff::civil::date;

use super::super::*;
use super::{config, launcher, remote};
use crate::ports::Repositories;
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
