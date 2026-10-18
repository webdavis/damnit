use super::super::*;
use super::{clock, committed_task, config, launcher};
use crate::ports::Repositories;
use crate::remote::{MutationOp, MutationOutcome, RemoteMutation};
use crate::testing::prelude::*;
use crate::testing::{EchoCredentials, FixedClock, FixedRandom, MemoryStore, oid};
use crate::use_cases::commit::commit;
use crate::use_cases::stage::add_all;
use dam_domain::{Field, Object, Task};
use jiff::civil::date;

fn answering(ok: bool) -> impl Fn(&[RemoteMutation]) -> Vec<MutationOutcome> + Clone {
    move |ms| {
        ms.iter()
            .map(|m| MutationOutcome {
                oid: m.oid.clone(),
                ok,
                remote_id: Some("r1".into()),
                why: (!ok).then(|| "temporary failure".to_string()),
            })
            .collect()
    }
}

#[test]
fn a_retry_declares_only_the_fields_of_the_change_that_failed() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    committed_task(&store, 1);
    let (l, _) = launcher(answering(true));
    push(repos, &l, &EchoCredentials, &clock(), &config(), None).unwrap();

    store
        .put(&Object::Task(Task::new(oid(1), "renamed")))
        .unwrap();
    add_all(&store, &store, &store).unwrap();
    commit(
        &store,
        &store,
        &FixedClock(date(2026, 9, 19)),
        &FixedRandom::new(2),
        "rename",
    )
    .unwrap();
    let (l, _) = launcher(answering(false));
    let reports = push(repos, &l, &EchoCredentials, &clock(), &config(), None).unwrap();
    assert_eq!(reports[0].failed.len(), 1);

    let (l, pushed) = launcher(answering(true));
    let reports = push(repos, &l, &EchoCredentials, &clock(), &config(), None).unwrap();
    assert_eq!(reports[0].succeeded, 1);
    let sent = pushed.borrow();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].op, MutationOp::Update);
    assert_eq!(sent[0].fields, vec![Field::Subject]);
}

#[test]
fn a_retry_keeps_the_failed_field_when_a_later_commit_changes_another() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    committed_task(&store, 1);
    let (l, _) = launcher(answering(true));
    push(repos, &l, &EchoCredentials, &clock(), &config(), None).unwrap();

    store
        .put(&Object::Task(Task::new(oid(1), "renamed")))
        .unwrap();
    add_all(&store, &store, &store).unwrap();
    commit(
        &store,
        &store,
        &FixedClock(date(2026, 9, 19)),
        &FixedRandom::new(2),
        "rename",
    )
    .unwrap();
    let (l, _) = launcher(answering(false));
    push(repos, &l, &EchoCredentials, &clock(), &config(), None).unwrap();

    let mut done = Task::new(oid(1), "renamed");
    done.done = true;
    store.put(&Object::Task(done)).unwrap();
    add_all(&store, &store, &store).unwrap();
    commit(
        &store,
        &store,
        &FixedClock(date(2026, 9, 20)),
        &FixedRandom::new(3),
        "done",
    )
    .unwrap();
    let (l, pushed) = launcher(answering(true));
    let reports = push(repos, &l, &EchoCredentials, &clock(), &config(), None).unwrap();
    assert_eq!(reports[0].succeeded, 1);
    let sent = pushed.borrow();
    assert_eq!(sent[0].fields, vec![Field::Subject, Field::Done]);
}
