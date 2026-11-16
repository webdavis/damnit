use dam_domain::{Completion, Object};

use super::{every_hook, retitled};
use crate::testing::{oid, task_in};

fn blocked_handed(dependency_done: bool) -> Option<bool> {
    let s = every_hook();
    let mut report = task_in(2, "report", "work");
    if dependency_done {
        report.completion = Completion::Done { at: None };
    }
    let mut laundry = task_in(3, "laundry", "home");
    laundry.base.depends.push(oid(2));
    s.committed(Object::Task(report));
    s.committed(Object::Task(laundry));
    s.dam().edit(&oid(3), retitled("fold laundry")).unwrap();
    let handed = s.fixture.hooks.handed.borrow();
    handed.last().unwrap().blocked
}

#[test]
fn a_hook_is_handed_blocked_as_dam_list_reads_it() {
    assert_eq!(blocked_handed(false), Some(true));
    assert_eq!(blocked_handed(true), Some(false));
}

#[test]
fn a_hook_is_handed_the_open_children_of_its_task() {
    let s = every_hook();
    s.committed(Object::Task(task_in(1, "trip", "home")));
    let mut pack = task_in(2, "pack", "home");
    pack.base.parent = Some(oid(1));
    s.committed(Object::Task(pack));
    s.dam().edit(&oid(1), retitled("big trip")).unwrap();
    let handed = s.fixture.hooks.handed.borrow();
    assert_eq!(handed.last().unwrap().children_open, Some(1));
}
