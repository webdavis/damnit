use dam_domain::Object;

use crate::errors::{Refusal, UseCaseError};
use crate::ports::ObjectRepository;
use crate::testing::{Scenario, oid, task_in};

fn refused_over_an_unsettled_task(s: &Scenario) -> UseCaseError {
    s.filed(task_in(5, "Home", ""))
        .filed(task_in(1, "errands", "Home"))
        .filed(task_in(2, "errands", "Home"));
    s.store
        .put(&Object::Task(task_in(9, "bins", "Home/errands")))
        .unwrap();
    s.dam().remove(&oid(1)).unwrap_err()
}

fn held_by_a_remote(error: UseCaseError) -> bool {
    match error {
        UseCaseError::Refused(Refusal::ParentNotKnown {
            held_by_a_remote, ..
        }) => held_by_a_remote,
        other => panic!("not a parent refusal: {other:?}"),
    }
}

#[test]
fn a_refusal_says_a_remote_holds_the_unsettled_object_it_has_an_id_for() {
    let s = Scenario::default().drawing_from(42);
    let refused = refused_over_an_unsettled_task(&s);
    s.mapped_as(&oid(9), "r9");
    assert!(held_by_a_remote(
        s.dam().telling_whether_a_remote_holds(refused)
    ));
}

#[test]
fn a_refusal_says_no_remote_holds_an_unsettled_object_no_remote_has_an_id_for() {
    let s = Scenario::default().drawing_from(42);
    let refused = refused_over_an_unsettled_task(&s);
    assert!(!held_by_a_remote(
        s.dam().telling_whether_a_remote_holds(refused)
    ));
}
