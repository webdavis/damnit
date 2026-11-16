use super::*;
use crate::{Object, Oid, Path, Task};

fn oid(b: u8) -> Oid {
    Oid::generate(&mut |x: &mut [u8]| x.fill(b))
}

fn task(b: u8, subject: &str, path: &str) -> Object {
    let mut t = Task::new(oid(b), subject);
    t.base.path = Path::parse(path).unwrap();
    Object::Task(t)
}

fn nearest(path: &str, objects: &[Object]) -> Vec<Oid> {
    nearest_parents(&Path::parse(path).unwrap(), objects)
}

#[test]
fn a_path_no_folder_holds_has_no_parent() {
    let objects = [task(1, "errands", "Home"), task(2, "milk", "Home/errands")];
    assert_eq!(nearest("Work", &objects), Vec::<Oid>::new());
    assert_eq!(nearest("Home", &objects), Vec::<Oid>::new());
}

#[test]
fn the_object_whose_folder_holds_the_path_is_its_parent() {
    let objects = [task(1, "errands", "Home"), task(2, "milk", "Home/errands")];
    assert_eq!(nearest("Home/errands", &objects), vec![oid(1)]);
}

#[test]
fn two_same_named_objects_both_hold_the_path() {
    let objects = [task(1, "errands", "Home"), task(2, "errands", "Home")];
    assert_eq!(nearest("Home/errands", &objects), vec![oid(1), oid(2)]);
}

#[test]
fn a_plain_place_below_an_object_is_held_by_that_object() {
    let objects = [task(1, "errands", "Home")];
    assert_eq!(nearest("Home/errands/later", &objects), vec![oid(1)]);
}

#[test]
fn the_nearest_of_two_nested_objects_wins() {
    let objects = [task(1, "errands", "Home"), task(2, "shop", "Home/errands")];
    assert_eq!(nearest("Home/errands/shop/later", &objects), vec![oid(2)]);
}

fn parented(b: u8, subject: &str, path: &str, parent: u8) -> Object {
    let mut object = task(b, subject, path);
    object.base_mut().parent = Some(oid(parent));
    object
}

#[test]
fn objects_whose_parents_are_known_are_settled() {
    let objects = [
        task(1, "errands", "Home"),
        parented(2, "milk", "Home/errands", 1),
    ];
    assert!(unsettled(&objects).is_empty());
}

#[test]
fn an_object_under_two_same_named_objects_is_unsettled() {
    let objects = [
        task(1, "errands", "Home"),
        task(2, "errands", "Home"),
        task(3, "milk", "Home/errands"),
    ];
    let found: Vec<_> = unsettled(&objects)
        .into_iter()
        .map(|(object, could_hold)| (object.oid().clone(), could_hold))
        .collect();
    assert_eq!(found, vec![(oid(3), vec![oid(1), oid(2)])]);
}

#[test]
fn an_object_no_folder_holds_is_settled_without_a_parent() {
    let objects = [task(1, "errands", "Home"), task(2, "milk", "Home")];
    assert!(unsettled(&objects).is_empty());
}

#[test]
fn an_object_in_a_plain_place_below_its_parent_is_settled() {
    let objects = [
        task(1, "errands", "Home"),
        parented(2, "milk", "Home/errands/later", 1),
    ];
    assert!(unsettled(&objects).is_empty());
}
