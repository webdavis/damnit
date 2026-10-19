use dam_domain::{Object, Task};

use super::super::*;
use super::{clock, config, launcher};
use crate::ports::{RemoteName, Repositories};
use crate::remote::{MutationOutcome, MutationVerdict};
use crate::service::Dam;
use crate::testing::prelude::*;
use crate::testing::{EchoCredentials, FixedRandom, Fixture, MemoryStore, oid};

#[test]
fn a_successful_delete_clears_the_remote_mapping() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    let remote = RemoteName("todoist".into());
    store.put(&Object::Task(Task::new(oid(1), "t1"))).unwrap();
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    dam.add_all().unwrap();
    let created = Dam {
        random: &FixedRandom::new(1),
        ..dam
    }
    .commit("m")
    .unwrap();
    store.map_remote_id(&remote, &oid(1), "r1").unwrap();
    store
        .set_remote_snapshot(&remote, &Object::Task(Task::new(oid(1), "t1")))
        .unwrap();
    store.mark_pushed(&remote, &created.id).unwrap();

    store.delete(&oid(1)).unwrap();
    dam.add_all().unwrap();
    Dam {
        random: &FixedRandom::new(2),
        ..dam
    }
    .commit("rm")
    .unwrap();

    let (l, _) = launcher(|ms| {
        ms.iter()
            .map(|m| MutationOutcome {
                oid: m.oid.clone(),
                verdict: MutationVerdict::Accepted { remote_id: None },
            })
            .collect()
    });
    push(repos, &l, &EchoCredentials, &clock(), &config(), None).unwrap();
    assert!(store.remote_id(&remote, &oid(1)).unwrap().is_none());
    assert!(store.remote_snapshot(&remote, &oid(1)).unwrap().is_none());
}

#[test]
fn a_delete_with_no_remote_mapping_is_skipped() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    store.put(&Object::Task(Task::new(oid(1), "t1"))).unwrap();
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    dam.add_all().unwrap();
    let created = Dam {
        random: &FixedRandom::new(1),
        ..dam
    }
    .commit("m")
    .unwrap();
    store
        .mark_pushed(&RemoteName("todoist".into()), &created.id)
        .unwrap();

    store.delete(&oid(1)).unwrap();
    dam.add_all().unwrap();
    Dam {
        random: &FixedRandom::new(2),
        ..dam
    }
    .commit("rm")
    .unwrap();

    let (l, pushed) = launcher(|_| vec![]);
    let reports = push(repos, &l, &EchoCredentials, &clock(), &config(), None).unwrap();
    assert_eq!(reports[0].skipped, 1);
    assert_eq!(reports[0].sent, 0);
    assert!(pushed.borrow().is_empty());
}
