use std::collections::BTreeSet;

use dam_domain::{Field, Object, Task};

use crate::ports::DoubtRepository;
use crate::remote::MutationOp;
use crate::testing::{
    Scenario, accepting_all, oid, pushing, remotes_launched_by, task, task_caps, todoist,
};

#[test]
fn a_create_accepted_while_in_doubt_is_followed_by_an_update_of_every_field_in_doubt() {
    let s = Scenario::default();
    s.committed(task(1));
    let (l, _) = pushing(|_| vec![]);
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    s.committed(Object::Task(Task::new(oid(1), "edited")));

    let (l, pushed) = pushing(accepting_all(Some("r1")));
    let report = s
        .dam()
        .push(remotes_launched_by(&l), None)
        .unwrap()
        .remove(0);

    let sent = pushed.borrow();
    let ops: Vec<MutationOp> = sent.iter().map(|m| m.op).collect();
    assert_eq!(
        ops,
        vec![MutationOp::Create, MutationOp::Update],
        "{sent:?}"
    );
    let every_field: BTreeSet<Field> = task_caps().fields.into_iter().collect();
    let declared: BTreeSet<Field> = sent[1].fields.iter().copied().collect();
    assert_eq!(declared, every_field);
    assert_eq!(sent[1].remote_id.as_deref(), Some("r1"));
    assert_ne!(sent[0].idempotency_key, sent[1].idempotency_key);
    assert_eq!((report.sent, report.succeeded), (2, 2));
    assert!(s.store.fields_in_doubt(&todoist()).unwrap().is_empty());
}

#[test]
fn a_create_accepted_with_nothing_in_doubt_is_not_followed_up() {
    let s = Scenario::default();
    s.committed(task(1));
    let (l, pushed) = pushing(accepting_all(Some("r1")));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    assert_eq!(pushed.borrow().len(), 1);
}
