use jiff::civil::date;

use super::super::*;
use super::{config, incoming, launcher, remote};
use crate::ports::Notice;
use crate::remote::{PullOutcome, RejectedObject};
use crate::testing::prelude::*;
use crate::testing::{FixedClock, FixedRandom, MemoryStore, remotes_launched_by};

fn rejecting(remote_id: &str, why: &str) -> PullOutcome {
    PullOutcome {
        rejected: vec![RejectedObject {
            remote_id: remote_id.into(),
            why: why.into(),
        }],
        ..PullOutcome::default()
    }
}

fn pull_from(store: &MemoryStore, answer: PullOutcome) {
    Dam {
        store,
        clock: &FixedClock(date(2026, 9, 18)),
        random: &FixedRandom::new(1),
        config: &config(),
    }
    .pull(remotes_launched_by(&launcher(answer)), None)
    .unwrap();
}

fn skipped(remote_id: &str, why: &str) -> Notice {
    Notice::PullSkipped {
        remote: remote(),
        object: RejectedObject {
            remote_id: remote_id.into(),
            why: why.into(),
        },
    }
}

#[test]
fn the_same_object_skipped_for_the_same_reason_twice_is_noticed_once() {
    let store = MemoryStore::new();
    pull_from(&store, rejecting("up-7", "its parent chain loops"));
    pull_from(&store, rejecting("up-7", "its parent chain loops"));
    assert_eq!(
        store.notices().unwrap(),
        vec![skipped("up-7", "its parent chain loops")]
    );
}

#[test]
fn the_same_object_skipped_for_a_new_reason_is_noticed_again() {
    let store = MemoryStore::new();
    pull_from(&store, rejecting("up-7", "its parent chain loops"));
    pull_from(&store, rejecting("up-7", "due: not a date"));
    assert_eq!(
        store.notices().unwrap(),
        vec![
            skipped("up-7", "its parent chain loops"),
            skipped("up-7", "due: not a date"),
        ]
    );
}

#[test]
fn an_object_that_read_cleanly_drops_its_skip_and_is_noticed_again_when_skipped_later() {
    let store = MemoryStore::new();
    pull_from(&store, rejecting("up-7", "its parent chain loops"));
    pull_from(
        &store,
        PullOutcome {
            objects: vec![incoming("from upstream", "up-7")],
            ..PullOutcome::default()
        },
    );
    assert_eq!(store.notices().unwrap(), vec![]);
    pull_from(&store, rejecting("up-7", "its parent chain loops"));
    assert_eq!(
        store.notices().unwrap(),
        vec![skipped("up-7", "its parent chain loops")]
    );
}
