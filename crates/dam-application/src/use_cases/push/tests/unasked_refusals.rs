use dam_domain::{Field, Object, Task};

use crate::ports::{
    AnswerRepository, AttemptRepository, CommitRepository, Notice, NoticeRepository,
};
use crate::remote::MutationOp;
use crate::testing::{
    Scenario, accepting_all, oid, pushing, refusing_all, refusing_all_unasked, remotes_launched_by,
    task, todoist,
};

const WHY: &str = "no Todoist project named work";

#[test]
fn a_refusal_the_remote_never_saw_is_retried_with_its_notice_and_leaves_the_count() {
    let s = Scenario::default();
    s.committed(task(1));
    let (l, _) = pushing(refusing_all_unasked(WHY));
    let report = s
        .dam()
        .push(remotes_launched_by(&l), None)
        .unwrap()
        .remove(0);
    assert_eq!(report.failed, vec![(oid(1), WHY.to_string())]);
    assert_eq!(s.store.push_retries(&todoist()).unwrap(), vec![oid(1)]);
    assert!(s.store.notices().unwrap().contains(&Notice::PushFailed {
        remote: todoist(),
        oid: oid(1),
        why: WHY.into(),
    }));
    assert_eq!(s.store.answers(&todoist(), &oid(1)).unwrap(), 0);
}

#[test]
fn a_refusal_the_remote_gave_moves_the_count() {
    let s = Scenario::default();
    s.committed(task(1));
    let (l, _) = pushing(refusing_all(WHY));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    assert_eq!(s.store.answers(&todoist(), &oid(1)).unwrap(), 1);
}

#[test]
fn an_unanswered_create_keeps_its_key_through_a_refusal_the_remote_never_gave() {
    let s = Scenario::default();
    s.committed(task(1));
    let (l, first) = pushing(|_| vec![]);
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    let (l, _) = pushing(refusing_all_unasked(WHY));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    let (l, third) = pushing(|_| vec![]);
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    let first = &first.borrow()[0];
    let third = &third.borrow()[0];
    assert_eq!(third.op, MutationOp::Create);
    assert_eq!(
        third.idempotency_key, first.idempotency_key,
        "the first create may have landed, and a new key would land it twice"
    );
}

fn recorded_keys(s: &Scenario) -> Vec<String> {
    let mut record = s.store.attempts(&todoist()).unwrap();
    let attempts = record.remove(&oid(1)).unwrap_or_default();
    attempts.into_iter().map(|a| a.key).collect()
}

#[test]
fn a_refusal_the_remote_never_saw_drops_the_attempt_it_added_and_keeps_the_earlier_one() {
    let s = Scenario::default();
    s.committed(task(1)).tracked_as(&oid(1), "r1").pushed();
    s.committed(Object::Task(Task::new(oid(1), "x")));
    let (l, unanswered) = pushing(|_| vec![]);
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    s.committed(Object::Task(Task::new(oid(1), "y")));
    let (l, _) = pushing(refusing_all_unasked(WHY));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    assert_eq!(
        recorded_keys(&s),
        vec![unanswered.borrow()[0].idempotency_key.clone()]
    );
}

#[test]
fn a_refusal_the_remote_never_saw_keeps_an_attempt_sent_before() {
    let s = Scenario::default();
    s.committed(task(1));
    let (l, unanswered) = pushing(|_| vec![]);
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    let (l, _) = pushing(refusing_all_unasked(WHY));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    assert_eq!(
        recorded_keys(&s),
        vec![unanswered.borrow()[0].idempotency_key.clone()]
    );
}

fn named(subject: &str) -> Object {
    Object::Task(Task::new(oid(1), subject))
}

fn push_with_no_answer(s: &Scenario) {
    let (l, _) = pushing(|_| vec![]);
    s.dam().push(remotes_launched_by(&l), None).unwrap();
}

#[test]
fn a_refusal_the_remote_never_saw_drops_only_the_create_it_added() {
    let s = Scenario::default();
    s.committed(named("X"));
    push_with_no_answer(&s);
    s.committed(named("Y"));
    push_with_no_answer(&s);
    s.committed(named("X"));
    let (l, _) = pushing(refusing_all_unasked(WHY));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    s.committed(named("Y"));
    let (l, pushed) = pushing(accepting_all(Some("r1")));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    let written: Vec<(MutationOp, Vec<Field>)> = pushed
        .borrow()
        .iter()
        .map(|m| (m.op, m.fields.clone()))
        .collect();
    assert_eq!(
        written[1..],
        [(MutationOp::Update, vec![Field::Subject])],
        "the first create sent X and may be what the remote holds"
    );
}
