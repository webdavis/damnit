use crate::ports::{AnswerRepository, CommitRepository, Notice, NoticeRepository};
use crate::remote::MutationOp;
use crate::testing::{
    Scenario, oid, pushing, refusing_all, refusing_all_unasked, remotes_launched_by, task, todoist,
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
fn a_create_in_doubt_keeps_its_key_through_a_refusal_the_remote_never_gave() {
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
