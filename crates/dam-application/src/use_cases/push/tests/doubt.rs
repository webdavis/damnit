use std::collections::{BTreeMap, BTreeSet};

use dam_domain::{Completion, Field, Object, Task};

use crate::ports::DoubtRepository;
use crate::remote::MutationOp;
use crate::testing::{
    Scenario, accepting_all, oid, pushing, remotes_launched_by, task, task_caps, todoist,
};

fn held_upstream_as_t1() -> Scenario {
    let s = Scenario::default();
    s.committed(task(1)).tracked_as(&oid(1), "r1").pushed();
    s
}

fn renamed(subject: &str) -> Task {
    Task::new(oid(1), subject)
}

fn push_with_no_answer(s: &Scenario) {
    let (l, _) = pushing(|_| vec![]);
    s.dam().push(remotes_launched_by(&l), None).unwrap();
}

fn in_doubt(s: &Scenario) -> BTreeMap<dam_domain::Oid, BTreeSet<Field>> {
    s.store.fields_in_doubt(&todoist()).unwrap()
}

#[test]
fn a_field_reverted_after_its_unanswered_update_is_still_sent() {
    let s = held_upstream_as_t1();
    s.committed(Object::Task(renamed("x")));
    push_with_no_answer(&s);
    s.committed(task(1));
    let (l, pushed) = pushing(accepting_all(None));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    let sent = pushed.borrow();
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert_eq!(sent[0].op, MutationOp::Update);
    assert_eq!(sent[0].fields, vec![Field::Subject]);
    assert_eq!(sent[0].object, Some(task(1)));
    assert!(in_doubt(&s).is_empty(), "{:?}", in_doubt(&s));
}

#[test]
fn the_next_push_declares_the_field_in_doubt_and_its_own_difference_only() {
    let s = held_upstream_as_t1();
    s.committed(Object::Task(renamed("x")));
    push_with_no_answer(&s);
    let mut done = renamed("t1");
    done.completion = Completion::Done { at: None };
    s.committed(Object::Task(done));
    let (l, pushed) = pushing(accepting_all(None));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    let sent = pushed.borrow();
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert_eq!(sent[0].fields, vec![Field::Subject, Field::Done]);
}

#[test]
fn an_unanswered_create_puts_every_field_the_remote_takes_in_doubt() {
    let s = Scenario::default();
    s.committed(task(1));
    push_with_no_answer(&s);
    let every_field: BTreeSet<Field> = task_caps().fields.into_iter().collect();
    assert_eq!(in_doubt(&s), BTreeMap::from([(oid(1), every_field)]));
}

#[test]
fn an_accepted_write_clears_the_fields_it_declares_and_leaves_the_rest() {
    let s = held_upstream_as_t1();
    let doubted = BTreeSet::from([Field::Subject, Field::Estimate]);
    s.store
        .set_fields_in_doubt(&todoist(), &oid(1), &doubted)
        .unwrap();
    let (l, pushed) = pushing(accepting_all(None));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    assert_eq!(pushed.borrow()[0].fields, vec![Field::Subject]);
    assert_eq!(
        in_doubt(&s),
        BTreeMap::from([(oid(1), BTreeSet::from([Field::Estimate]))])
    );
}
