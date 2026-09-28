use jiff::civil::date;

use super::super::*;
use super::{clock, committed_task, config, launcher};
use crate::ports::Repositories;
use crate::remote::{MutationOp, MutationOutcome};
use crate::testing::prelude::*;
use crate::testing::{EchoCredentials, FixedClock, FixedRandom, MemoryStore, oid};
use crate::use_cases::commit::commit;
use crate::use_cases::stage::add_all;
use dam_domain::{Object, Task};

#[test]
fn a_create_the_remote_already_holds_is_sent_as_an_update() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    committed_task(&store, 1);
    store
        .map_remote_id(&RemoteName("todoist".into()), &oid(1), "r1")
        .unwrap();
    let (l, pushed) = launcher(|_| vec![]);
    push(repos, &l, &EchoCredentials, &clock(), &config(), None).unwrap();
    assert_eq!(pushed.borrow()[0].op, MutationOp::Update);
}

#[test]
fn an_update_of_an_object_the_remote_never_took_is_sent_as_a_create() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    committed_task(&store, 1);
    let (l, pushed) = launcher(|ms| {
        ms.iter()
            .map(|m| MutationOutcome {
                oid: m.oid.clone(),
                ok: true,
                remote_id: None,
                why: None,
            })
            .collect()
    });
    push(repos, &l, &EchoCredentials, &clock(), &config(), None).unwrap();
    store
        .put(&Object::Task(Task::new(oid(1), "edited")))
        .unwrap();
    add_all(&store, &store, &store).unwrap();
    commit(
        &store,
        &store,
        &FixedClock(date(2026, 9, 19)),
        &FixedRandom::new(9),
        "an update the remote has never seen the object for",
    )
    .unwrap();
    push(repos, &l, &EchoCredentials, &clock(), &config(), None).unwrap();
    let sent = pushed.borrow();
    assert_eq!(sent.len(), 2);
    assert_eq!(sent[1].op, MutationOp::Create);
}
