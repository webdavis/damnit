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
