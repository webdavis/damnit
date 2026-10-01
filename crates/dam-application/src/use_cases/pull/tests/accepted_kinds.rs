use dam_domain::Kind;

use super::{config, launcher, remote};
use crate::ports::Repositories;
use crate::remote::PullOutcome;
use crate::testing::prelude::*;
use crate::testing::{EchoCredentials, FixedClock, FixedRandom, MemoryStore};
use crate::use_cases::pull::pull;

#[test]
fn a_pull_records_the_kinds_the_remote_accepts() {
    let store = MemoryStore::new();
    pull(
        Repositories::of(&store),
        &launcher(PullOutcome::default()),
        &EchoCredentials,
        &FixedClock(jiff::civil::date(2026, 9, 18)),
        &FixedRandom::new(1),
        &config(),
        None,
    )
    .unwrap();
    assert_eq!(
        store.accepted_kinds(&remote()).unwrap(),
        Some(vec![Kind::Task])
    );
}
