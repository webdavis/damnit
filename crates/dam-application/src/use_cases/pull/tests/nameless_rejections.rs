use jiff::civil::date;

use super::super::*;
use super::{config, launcher};
use crate::ports::Repositories;
use crate::remote::{PullOutcome, RejectedObject};
use crate::testing::prelude::*;
use crate::testing::{EchoCredentials, FixedClock, FixedRandom, MemoryStore};

#[test]
fn a_rejection_with_no_remote_id_is_noticed_by_its_reason_alone() {
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
    let why: Vec<String> = store
        .notices()
        .unwrap()
        .into_iter()
        .filter_map(|n| match n {
            Notice::PullFailed { why, .. } => Some(why),
            _ => None,
        })
        .collect();
    assert_eq!(
        why,
        vec!["an object with no remote id was skipped".to_string()]
    );
}
