use super::*;
use crate::testing::{Scenario, oid, task_in};

#[test]
fn children_are_the_objects_in_the_folder_an_object_owns() {
    let s = Scenario::default();
    s.filed(task_in(1, "plan", "work"))
        .filed(task_in(2, "step", "work/plan"))
        .filed(task_in(3, "detail", "work/plan/step"))
        .filed(task_in(4, "note", "work"));
    let owner = s.store.get(&oid(1)).unwrap().unwrap();
    let found: Vec<Oid> = children(&s.store, owner.base())
        .unwrap()
        .iter()
        .map(|o| o.oid().clone())
        .collect();
    assert_eq!(found, vec![oid(2)]);
}

#[test]
fn moving_an_object_into_a_folder_carries_the_folder_it_owns() {
    let s = Scenario::default();
    s.filed(task_in(2, "step", "work/plan"))
        .filed(task_in(3, "detail", "work/plan/step"));
    move_into(&s.store, &oid(2), &Path::parse("work").unwrap()).unwrap();
    assert_eq!(s.path_of(&oid(2)), "work/");
    assert_eq!(s.path_of(&oid(3)), "work/step/");
}
