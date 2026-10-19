use dam_domain::Kind;

use super::{config, launcher, remote};
use crate::remote::PullOutcome;
use crate::service::Dam;
use crate::testing::prelude::*;
use crate::testing::{FixedClock, FixedRandom, MemoryStore, remotes_launched_by};

#[test]
fn a_pull_records_the_kinds_the_remote_accepts() {
    let store = MemoryStore::new();
    Dam {
        store: &store,
        clock: &FixedClock(jiff::civil::date(2026, 9, 18)),
        random: &FixedRandom::new(1),
        config: &config(),
    }
    .pull(remotes_launched_by(&launcher(PullOutcome::default())), None)
    .unwrap();
    assert_eq!(
        store.accepted_kinds(&remote()).unwrap(),
        Some(vec![Kind::Task])
    );
}
