use dam_domain::{Authored, Object};

use super::{dentist, retitled_by_tool};
use crate::ports::ObjectRepository;
use crate::testing::{Scenario, oid, task_in};
use crate::use_cases::prefix::Named;

fn staged_and_working_parents(
    s: &Scenario,
    staged: &[Authored],
) -> (Option<dam_domain::Oid>, Option<dam_domain::Oid>) {
    let after = staged[0].change.after().unwrap();
    let working = s.store.get(after.oid()).unwrap().unwrap();
    (after.base().parent.clone(), working.base().parent.clone())
}

#[test]
fn an_added_new_object_is_staged_with_the_parent_it_was_given() {
    let s = Scenario::default().drawing_from(7);
    let hash = dentist(&s);
    s.filed(task_in(5, "inbox", ""));
    let staged = s.dam().add(&[Named::Item(hash)]).unwrap().staged;
    let (staged, working) = staged_and_working_parents(&s, &staged);
    assert_eq!(working, Some(oid(5)));
    assert_eq!(staged, working);
}

#[test]
fn an_added_change_is_staged_with_the_parent_the_working_copy_keeps() {
    let s = Scenario::default().drawing_from(7);
    s.filed(task_in(5, "home", ""))
        .filed(task_in(6, "home", ""));
    s.committed(Object::Task(task_in(1, "pay rent", "home")));
    let mut working = task_in(1, "pay rent", "home");
    working.base.parent = Some(oid(6));
    s.filed(working);
    let hash = retitled_by_tool(&s, "pay the rent");
    let staged = s.dam().add(&[Named::Item(hash)]).unwrap().staged;
    let (staged, working) = staged_and_working_parents(&s, &staged);
    assert_eq!(working, Some(oid(6)));
    assert_eq!(staged, working);
}
