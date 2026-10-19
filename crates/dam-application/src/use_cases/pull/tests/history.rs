use jiff::civil::date;

use super::super::*;
use super::{config, incoming, launcher, remote};
use crate::remote::PullOutcome;
use crate::testing::prelude::*;
use crate::testing::{FixedClock, FixedRandom, Fixture, MemoryStore, oid, remotes_launched_by};
use dam_domain::{Object, Task};

#[test]
fn a_pulled_update_records_the_object_it_replaced() {
    let store = MemoryStore::new();
    let old = Object::Task(Task::new(oid(1), "old"));
    store.put(&old).unwrap();
    let fixture = Fixture {
        random: FixedRandom::new(9),
        ..Fixture::default()
    };
    let dam = fixture.dam(&store);
    dam.add_all().unwrap();
    dam.commit("m").unwrap();
    store.map_remote_id(&remote(), &oid(1), "r1").unwrap();
    store.set_remote_snapshot(&remote(), &old).unwrap();
    let l = launcher(PullOutcome {
        objects: vec![incoming("new", "r1")],
        ..PullOutcome::default()
    });
    Dam {
        store: &store,
        clock: &FixedClock(date(2026, 9, 18)),
        random: &FixedRandom::new(42),
        config: &config(),
    }
    .pull(remotes_launched_by(&l), None)
    .unwrap();
    let pulled = &store.log().unwrap()[0].changes[0];
    assert_eq!(pulled.before(), Some(&old));
}
