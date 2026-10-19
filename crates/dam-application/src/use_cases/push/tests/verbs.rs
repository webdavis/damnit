use crate::remote::MutationOp;
use crate::testing::prelude::*;
use crate::testing::{Scenario, accepting_all, oid, pushing, remotes_launched_by, task, todoist};
use dam_domain::{Object, Task};

#[test]
fn a_create_the_remote_already_holds_is_sent_as_an_update() {
    let s = Scenario::default();
    s.committed(task(1));
    s.store.map_remote_id(&todoist(), &oid(1), "r1").unwrap();
    let (l, pushed) = pushing(|_| vec![]);
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    assert_eq!(pushed.borrow()[0].op, MutationOp::Update);
}

#[test]
fn an_update_of_an_object_the_remote_never_took_is_sent_as_a_create() {
    let s = Scenario::default();
    s.committed(task(1));
    let (l, pushed) = pushing(accepting_all(None));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    s.committed(Object::Task(Task::new(oid(1), "edited")));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    let sent = pushed.borrow();
    assert_eq!(sent.len(), 2);
    assert_eq!(sent[1].op, MutationOp::Create);
}

#[test]
fn an_edit_and_its_undo_pushed_together_send_nothing() {
    let s = Scenario::default();
    s.committed(task(1));
    let (l, pushed) = pushing(accepting_all(None));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    s.committed(Object::Task(Task::new(oid(1), "edited")))
        .committed(task(1));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    assert_eq!(
        pushed.borrow().len(),
        1,
        "only the first push sent anything"
    );
}
