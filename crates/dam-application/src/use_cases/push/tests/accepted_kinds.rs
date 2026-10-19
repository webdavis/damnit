use dam_domain::Kind;

use super::super::*;
use super::{clock, config, launcher};
use crate::testing::prelude::*;
use crate::testing::{FixedRandom, MemoryStore, remotes_launched_by};

#[test]
fn a_push_records_the_kinds_the_remote_accepts() {
    let store = MemoryStore::new();
    let (l, _) = launcher(|_| vec![]);
    Dam {
        store: &store,
        clock: &clock(),
        random: &FixedRandom::new(0),
        config: &config(),
    }
    .push(remotes_launched_by(&l), None)
    .unwrap();
    assert_eq!(
        store.accepted_kinds(&RemoteName("todoist".into())).unwrap(),
        Some(vec![Kind::Task])
    );
}
