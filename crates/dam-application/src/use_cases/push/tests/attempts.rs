use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

use dam_domain::{Field, Object, Task};

use super::order::held_upstream;
use crate::ports::{Attempt, AttemptRepository, HelperError, ObjectRepository};
use crate::remote::{MutationOp, PushOutcome};
use crate::testing::{
    Fixture, MemoryStore, Scenario, ScriptedHelper, ScriptedLauncher, accepting_all, config, oid,
    pushing, refusing_all, remotes_launched_by, task, task_caps, task_in, todoist,
};

fn recorded(store: &dyn AttemptRepository, byte: u8) -> Vec<Attempt> {
    let mut record = store.attempts(&todoist()).unwrap();
    record.remove(&oid(byte)).unwrap_or_default()
}

fn ops(attempts: &[Attempt]) -> Vec<MutationOp> {
    attempts.iter().map(|a| a.op).collect()
}

fn push_with_no_answer(s: &Scenario) {
    let (l, _) = pushing(|_| vec![]);
    s.dam().push(remotes_launched_by(&l), None).unwrap();
}

fn timing_out() -> ScriptedLauncher {
    ScriptedLauncher::launching(|| ScriptedHelper {
        push_answer: Box::new(|_| Err(HelperError::Cancelled)),
        ..ScriptedHelper::answering(Ok(Default::default()))
    })
}

#[test]
fn the_attempt_is_in_the_store_when_the_helper_is_called() {
    let store: &'static MemoryStore = Box::leak(Box::default());
    let fixture = Fixture {
        config: config(),
        ..Fixture::default()
    };
    store.put(&task(1)).unwrap();
    fixture.dam(store).add_all().unwrap();
    fixture.dam(store).commit("m").unwrap();
    let seen = Rc::new(RefCell::new(None));
    let saw = seen.clone();
    let launcher = ScriptedLauncher::launching(move || {
        let saw = saw.clone();
        ScriptedHelper {
            push_answer: Box::new(move |_| {
                saw.replace(Some(ops(&recorded(store, 1))));
                Ok(PushOutcome::default())
            }),
            ..ScriptedHelper::answering(Ok(Default::default()))
        }
    });
    fixture
        .dam(store)
        .push(remotes_launched_by(&launcher), None)
        .unwrap();
    assert_eq!(seen.take(), Some(vec![MutationOp::Create]));
}

#[test]
fn an_unanswered_create_stays_recorded_with_its_key_and_every_field_it_sent() {
    let s = Scenario::default();
    s.committed(task(1));
    let (l, pushed) = pushing(|_| vec![]);
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    let attempts = recorded(&s.store, 1);
    assert_eq!(ops(&attempts), vec![MutationOp::Create]);
    assert_eq!(attempts[0].key, pushed.borrow()[0].idempotency_key);
    let declared: BTreeSet<Field> = attempts[0].declared.keys().copied().collect();
    assert_eq!(declared, task_caps().fields.into_iter().collect());
}

#[test]
fn an_accepted_write_clears_its_object_s_attempts() {
    let s = Scenario::default();
    s.committed(task(1));
    push_with_no_answer(&s);
    let (l, _) = pushing(accepting_all(Some("r1")));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    assert!(s.store.attempts(&todoist()).unwrap().is_empty());
}

#[test]
fn a_refused_write_drops_its_own_attempt_and_keeps_the_earlier_one() {
    let s = Scenario::default();
    s.committed(task(1)).tracked_as(&oid(1), "r1").pushed();
    s.committed(Object::Task(Task::new(oid(1), "x")));
    let (l, unanswered) = pushing(|_| vec![]);
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    s.committed(Object::Task(Task::new(oid(1), "y")));
    let (l, refused) = pushing(refusing_all("not now"));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    let keys: Vec<String> = recorded(&s.store, 1).into_iter().map(|a| a.key).collect();
    assert_ne!(
        refused.borrow()[0].idempotency_key,
        unanswered.borrow()[0].idempotency_key
    );
    assert_eq!(keys, vec![unanswered.borrow()[0].idempotency_key.clone()]);
}

#[test]
fn a_write_of_a_later_wave_never_handed_to_the_helper_records_no_attempt() {
    let s = held_upstream(&[task_in(1, "P", "Home/"), task_in(2, "c", "Home/P/")]);
    s.filed(task_in(2, "c", "Home/")).deleted(&oid(1));
    s.dam()
        .push(remotes_launched_by(&timing_out()), None)
        .unwrap_err();
    assert_eq!(ops(&recorded(&s.store, 2)), vec![MutationOp::Update]);
    assert!(recorded(&s.store, 1).is_empty());
}
