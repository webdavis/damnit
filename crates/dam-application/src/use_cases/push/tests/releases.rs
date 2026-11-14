use dam_domain::{Completion, Path, Task};

use super::namesakes::why_held;
use super::order::held_upstream;
use crate::remote::MutationOp;
use crate::testing::Scenario;
use crate::testing::{
    accepting_all, config, oid, pushing_to_a_remote_that_removes_only_empty_containers,
    remotes_launched_by, task_in,
};

fn done(mut t: Task) -> Task {
    t.completion = Completion::Done { at: None };
    t
}

fn sent_to_a_remote_that_removes_only_empty_containers(s: &Scenario) -> Vec<MutationOp> {
    let (l, pushed) =
        pushing_to_a_remote_that_removes_only_empty_containers(accepting_all(Some("i:new")));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    pushed.borrow().iter().map(|m| m.op).collect()
}

#[test]
fn a_section_whose_done_task_left_the_remote_path_stays_held_beside_a_namesake() {
    for (removed, namesake, moved) in [(1, 2, 3), (3, 2, 1), (2, 3, 1)] {
        let mut scoped = config();
        scoped.remotes[0].path = Some(Path::parse("Home").unwrap());
        let s = held_upstream(&[
            task_in(removed, "S", "Home/"),
            task_in(namesake, "S", "Home/"),
            done(task_in(moved, "T", "Home/S/")),
        ])
        .configured(scoped);
        s.filed(done(task_in(moved, "T", "Work/")))
            .deleted(&oid(removed));
        assert!(sent_to_a_remote_that_removes_only_empty_containers(&s).is_empty());
        let why = why_held(&s, &oid(removed));
        assert!(
            why.contains("is still under it upstream and this push does not move it"),
            "{why}"
        );
    }
}

#[test]
fn a_removal_beside_two_namesakes_that_stay_is_held_for_the_namesakes() {
    for (removed, first, second, child) in [(1, 2, 3, 4), (4, 3, 2, 1), (2, 4, 1, 3)] {
        let s = held_upstream(&[
            task_in(removed, "P", "Home/"),
            task_in(first, "P", "Home/"),
            task_in(second, "P", "Home/"),
            task_in(child, "c", "Home/P/"),
        ]);
        s.deleted(&oid(removed));
        assert!(sent_to_a_remote_that_removes_only_empty_containers(&s).is_empty());
        let why = why_held(&s, &oid(removed));
        assert!(why.contains("same name in the same place"), "{why}");
        assert!(why.contains(oid(child).short()), "{why}");
    }
}
