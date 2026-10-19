use super::{clock, committed_task, config, launcher};
use crate::remote::{MutationOp, MutationOutcome, MutationVerdict, RemoteMutation};
use crate::service::Dam;
use crate::testing::prelude::*;
use crate::testing::{FixedClock, FixedRandom, Fixture, MemoryStore, oid, remotes_launched_by};
use dam_domain::{Completion, Field, Object, Task};
use jiff::civil::date;

fn answering(ok: bool) -> impl Fn(&[RemoteMutation]) -> Vec<MutationOutcome> + Clone {
    move |ms| {
        ms.iter()
            .map(|m| MutationOutcome {
                oid: m.oid.clone(),
                verdict: match ok {
                    true => MutationVerdict::Accepted {
                        remote_id: Some("r1".into()),
                    },
                    false => MutationVerdict::Refused {
                        why: "temporary failure".into(),
                    },
                },
            })
            .collect()
    }
}

#[test]
fn a_retry_declares_only_the_fields_of_the_change_that_failed() {
    let store = MemoryStore::new();
    committed_task(&store, 1);
    let (l, _) = launcher(answering(true));
    Dam {
        store: &store,
        clock: &clock(),
        random: &FixedRandom::new(0),
        config: &config(),
    }
    .push(remotes_launched_by(&l), None)
    .unwrap();

    store
        .put(&Object::Task(Task::new(oid(1), "renamed")))
        .unwrap();
    let fixture = Fixture {
        clock: FixedClock(date(2026, 9, 19)),
        random: FixedRandom::new(2),
        ..Fixture::default()
    };
    let dam = fixture.dam(&store);
    dam.add_all().unwrap();
    dam.commit("rename").unwrap();
    let (l, _) = launcher(answering(false));
    let reports = Dam {
        store: &store,
        clock: &clock(),
        random: &FixedRandom::new(0),
        config: &config(),
    }
    .push(remotes_launched_by(&l), None)
    .unwrap();
    assert_eq!(reports[0].failed.len(), 1);

    let (l, pushed) = launcher(answering(true));
    let reports = Dam {
        store: &store,
        clock: &clock(),
        random: &FixedRandom::new(0),
        config: &config(),
    }
    .push(remotes_launched_by(&l), None)
    .unwrap();
    assert_eq!(reports[0].succeeded, 1);
    let sent = pushed.borrow();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].op, MutationOp::Update);
    assert_eq!(sent[0].fields, vec![Field::Subject]);
}

#[test]
fn a_retry_keeps_the_failed_field_when_a_later_commit_changes_another() {
    let store = MemoryStore::new();
    committed_task(&store, 1);
    let (l, _) = launcher(answering(true));
    Dam {
        store: &store,
        clock: &clock(),
        random: &FixedRandom::new(0),
        config: &config(),
    }
    .push(remotes_launched_by(&l), None)
    .unwrap();

    store
        .put(&Object::Task(Task::new(oid(1), "renamed")))
        .unwrap();
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    dam.add_all().unwrap();
    Dam {
        clock: &FixedClock(date(2026, 9, 19)),
        random: &FixedRandom::new(2),
        ..dam
    }
    .commit("rename")
    .unwrap();
    let (l, _) = launcher(answering(false));
    Dam {
        store: &store,
        clock: &clock(),
        random: &FixedRandom::new(0),
        config: &config(),
    }
    .push(remotes_launched_by(&l), None)
    .unwrap();

    let mut done = Task::new(oid(1), "renamed");
    done.completion = Completion::Done { at: None };
    store.put(&Object::Task(done)).unwrap();
    dam.add_all().unwrap();
    Dam {
        clock: &FixedClock(date(2026, 9, 20)),
        random: &FixedRandom::new(3),
        ..dam
    }
    .commit("done")
    .unwrap();
    let (l, pushed) = launcher(answering(true));
    let reports = Dam {
        store: &store,
        clock: &clock(),
        random: &FixedRandom::new(0),
        config: &config(),
    }
    .push(remotes_launched_by(&l), None)
    .unwrap();
    assert_eq!(reports[0].succeeded, 1);
    let sent = pushed.borrow();
    assert_eq!(sent[0].fields, vec![Field::Subject, Field::Done]);
}
