use dam_domain::{Object, Path, Task};

use crate::ports::{AttemptRepository, ObjectRepository};
use crate::remote::{IncomingObject, MutationOp, PullOutcome};
use crate::testing::{
    Scenario, accepting_all, config, oid, pulling, pushing, remotes_launched_by, task, task_in,
    todoist,
};

fn named(subject: &str) -> Object {
    Object::Task(Task::new(oid(1), subject))
}

fn keys_sent_with_no_answer(s: &Scenario) -> Vec<String> {
    let (l, pushed) = pushing(|_| vec![]);
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    let sent = pushed.borrow();
    sent.iter().map(|m| m.idempotency_key.clone()).collect()
}

fn push_with_no_answer(s: &Scenario) {
    keys_sent_with_no_answer(s);
}

fn push_accepting(s: &Scenario) -> (Vec<MutationOp>, usize) {
    let (l, pushed) = pushing(accepting_all(None));
    let report = s
        .dam()
        .push(remotes_launched_by(&l), None)
        .unwrap()
        .remove(0);
    let ops = pushed.borrow().iter().map(|m| m.op).collect();
    (ops, report.skipped)
}

fn pull_holding(s: &Scenario, subject: &str) {
    let l = pulling(PullOutcome {
        objects: vec![IncomingObject {
            remote_id: "r1".into(),
            object: named(subject),
        }],
        ..PullOutcome::default()
    });
    s.dam().pull(remotes_launched_by(&l), None).unwrap();
}

fn attempted(s: &Scenario) -> bool {
    s.store.attempts(&todoist()).unwrap().contains_key(&oid(1))
}

#[test]
fn a_record_with_nothing_left_to_declare_is_cleared_and_not_skipped() {
    let s = Scenario::default();
    s.committed(task(1)).tracked_as(&oid(1), "r1").pushed();
    s.committed(named("x"));
    push_with_no_answer(&s);
    pull_holding(&s, "x");

    assert_eq!(push_accepting(&s), (vec![], 0));
    assert!(!attempted(&s));
}

#[test]
fn a_write_after_a_cleared_record_gets_a_key_the_remote_has_not_run() {
    let s = Scenario::default();
    s.committed(task(1)).tracked_as(&oid(1), "r1").pushed();
    s.committed(named("x"));
    let unanswered = keys_sent_with_no_answer(&s);
    pull_holding(&s, "x");
    assert_eq!(push_accepting(&s), (vec![], 0));
    assert!(!attempted(&s));
    pull_holding(&s, "y");
    s.committed(named("x"));

    let after_the_record_was_cleared = keys_sent_with_no_answer(&s);

    assert_eq!(unanswered.len(), 1);
    assert_eq!(after_the_record_was_cleared.len(), 1);
    assert_ne!(unanswered, after_the_record_was_cleared);
}

#[test]
fn a_record_of_an_object_removed_here_that_sends_nothing_is_cleared() {
    let mut config = config();
    config.remotes[0].path = Some(Path::parse("Work").unwrap());
    let s = Scenario::default().configured(config);
    s.committed(Object::Task(task_in(1, "Ada", "Home/")))
        .tracked_as(&oid(1), "r1")
        .pushed();
    s.committed(Object::Task(task_in(1, "Ada", "Work/")));
    push_with_no_answer(&s);
    s.committed(Object::Task(task_in(1, "Ada", "Home/")));
    push_accepting(&s);
    s.deleted(&oid(1));

    assert_eq!(push_accepting(&s), (vec![], 0));
    assert!(!attempted(&s));
}

#[test]
fn a_delete_with_no_answer_then_set_back_keeps_its_record_and_a_pull_keeps_the_object() {
    let s = Scenario::default();
    s.committed(task(1)).tracked_as(&oid(1), "r1").pushed();
    s.deleted(&oid(1));
    push_with_no_answer(&s);
    s.committed(task(1));
    assert_eq!(push_accepting(&s), (vec![], 1));
    assert!(attempted(&s));

    let l = pulling(PullOutcome {
        removed: vec!["r1".into()],
        ..PullOutcome::default()
    });
    s.dam().pull(remotes_launched_by(&l), None).unwrap();
    assert_eq!(s.store.get(&oid(1)).unwrap(), Some(task(1)));
    assert_eq!(push_accepting(&s), (vec![MutationOp::Create], 0));
    assert!(!attempted(&s));
}
