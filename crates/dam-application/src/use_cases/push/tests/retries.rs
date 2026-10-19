use crate::remote::MutationOp;
use crate::testing::{
    Scenario, accepting_all, oid, pushing, refusing_all, remotes_launched_by, task,
};
use dam_domain::{Completion, Field, Object, Task};

#[test]
fn a_retry_declares_only_the_fields_of_the_change_that_failed() {
    let s = Scenario::default();
    s.committed(task(1));
    let (l, _) = pushing(accepting_all(Some("r1")));
    s.dam().push(remotes_launched_by(&l), None).unwrap();

    s.committed(Object::Task(Task::new(oid(1), "renamed")));
    let (l, _) = pushing(refusing_all("temporary failure"));
    let reports = s.dam().push(remotes_launched_by(&l), None).unwrap();
    assert_eq!(reports[0].failed.len(), 1);

    let (l, pushed) = pushing(accepting_all(Some("r1")));
    let reports = s.dam().push(remotes_launched_by(&l), None).unwrap();
    assert_eq!(reports[0].succeeded, 1);
    let sent = pushed.borrow();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].op, MutationOp::Update);
    assert_eq!(sent[0].fields, vec![Field::Subject]);
}

#[test]
fn a_retry_keeps_the_failed_field_when_a_later_commit_changes_another() {
    let s = Scenario::default();
    s.committed(task(1));
    let (l, _) = pushing(accepting_all(Some("r1")));
    s.dam().push(remotes_launched_by(&l), None).unwrap();

    s.committed(Object::Task(Task::new(oid(1), "renamed")));
    let (l, _) = pushing(refusing_all("temporary failure"));
    s.dam().push(remotes_launched_by(&l), None).unwrap();

    let mut done = Task::new(oid(1), "renamed");
    done.completion = Completion::Done { at: None };
    s.committed(Object::Task(done));
    let (l, pushed) = pushing(accepting_all(Some("r1")));
    let reports = s.dam().push(remotes_launched_by(&l), None).unwrap();
    assert_eq!(reports[0].succeeded, 1);
    let sent = pushed.borrow();
    assert_eq!(sent[0].fields, vec![Field::Subject, Field::Done]);
}
