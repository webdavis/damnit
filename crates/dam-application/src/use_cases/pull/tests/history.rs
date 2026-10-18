use jiff::civil::date;

use super::super::*;
use super::{config, incoming, launcher, remote};
use crate::ports::Repositories;
use crate::remote::PullOutcome;
use crate::testing::prelude::*;
use crate::testing::{EchoCredentials, FixedClock, FixedRandom, MemoryStore, oid};
use crate::use_cases::commit::commit;
use crate::use_cases::stage::add_all;
use dam_domain::{Object, Task};

#[test]
fn a_pulled_update_records_the_object_it_replaced() {
    let store = MemoryStore::new();
    let old = Object::Task(Task::new(oid(1), "old"));
    store.put(&old).unwrap();
    add_all(&store, &store, &store).unwrap();
    commit(
        &store,
        &store,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(9),
        "m",
    )
    .unwrap();
    store.map_remote_id(&remote(), &oid(1), "r1").unwrap();
    store.set_remote_snapshot(&remote(), &old).unwrap();
    let l = launcher(PullOutcome {
        objects: vec![incoming("new", "r1")],
        ..PullOutcome::default()
    });
    pull(
        Repositories::of(&store),
        &l,
        &EchoCredentials,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(42),
        &config(),
        None,
    )
    .unwrap();
    let pulled = &store.log().unwrap()[0].changes[0];
    assert_eq!(pulled.before(), Some(&old));
}
