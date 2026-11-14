use dam_domain::{Object, Oid, Task};

use crate::remote::{MutationOp, MutationOutcome, MutationVerdict, RemoteMutation};
use crate::testing::{
    Scenario, accepting_all, oid, pushing, refusing_all, remotes_launched_by, task_in,
};

use MutationOp::{Create, Delete, Update};

pub(super) const BOTH_OID_ORDERS: [(u8, u8); 2] = [(1, 2), (2, 1)];

pub(super) fn held_upstream(tasks: &[Task]) -> Scenario {
    let s = Scenario::default();
    for (i, t) in tasks.iter().enumerate() {
        s.committed(Object::Task(t.clone()))
            .tracked_as(&t.base.oid, &format!("i:{i}"));
    }
    s.pushed();
    s
}

pub(super) fn sent(s: &Scenario) -> Vec<(Oid, MutationOp)> {
    let (l, pushed) = pushing(accepting_all(Some("i:new")));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    pushed
        .borrow()
        .iter()
        .map(|m| (m.oid.clone(), m.op))
        .collect()
}

#[test]
fn a_child_moved_up_is_sent_before_its_parent_is_removed() {
    for (parent, child) in BOTH_OID_ORDERS {
        let s = held_upstream(&[
            task_in(parent, "P", "Home/"),
            task_in(child, "c", "Home/P/"),
        ]);
        s.filed(task_in(child, "c", "Home/")).deleted(&oid(parent));
        assert_eq!(sent(&s), vec![(oid(child), Update), (oid(parent), Delete)]);
    }
}

#[test]
fn a_child_removed_with_its_parent_is_removed_first() {
    for (parent, child) in BOTH_OID_ORDERS {
        let s = held_upstream(&[
            task_in(parent, "P", "Home/"),
            task_in(child, "c", "Home/P/"),
        ]);
        s.deleted(&oid(parent)).deleted(&oid(child));
        assert_eq!(sent(&s), vec![(oid(child), Delete), (oid(parent), Delete)]);
    }
}

#[test]
fn a_parent_is_created_before_the_child_placed_under_it() {
    for (parent, child) in BOTH_OID_ORDERS {
        let s = Scenario::default();
        s.filed(task_in(parent, "P", "Home/"))
            .committed(Object::Task(task_in(child, "c", "Home/P/")));
        assert_eq!(sent(&s), vec![(oid(parent), Create), (oid(child), Create)]);
    }
}

#[test]
fn a_parent_is_created_before_a_task_moves_under_it() {
    for (parent, moved) in BOTH_OID_ORDERS {
        let s = held_upstream(&[task_in(moved, "m", "Home/")]);
        s.filed(task_in(parent, "P", "Home/"))
            .committed(Object::Task(task_in(moved, "m", "Home/P/")));
        assert_eq!(sent(&s), vec![(oid(parent), Create), (oid(moved), Update)]);
    }
}

#[test]
fn a_section_is_removed_after_its_tasks_move_out() {
    for (section, moved) in BOTH_OID_ORDERS {
        let s = held_upstream(&[
            task_in(section, "S", "Home/"),
            task_in(moved, "m", "Home/S/"),
        ]);
        s.filed(task_in(moved, "m", "Home/")).deleted(&oid(section));
        assert_eq!(sent(&s), vec![(oid(moved), Update), (oid(section), Delete)]);
    }
}

#[test]
fn a_project_is_removed_after_its_tasks_move_to_another() {
    for (project, moved) in BOTH_OID_ORDERS {
        let s = held_upstream(&[task_in(project, "Home", ""), task_in(moved, "m", "Home/")]);
        s.filed(task_in(moved, "m", "Work/")).deleted(&oid(project));
        assert_eq!(sent(&s), vec![(oid(moved), Update), (oid(project), Delete)]);
    }
}

#[test]
fn a_project_is_created_before_its_section_and_the_section_before_its_task() {
    let s = Scenario::default();
    s.filed(task_in(1, "t", "New/S/"))
        .filed(task_in(2, "S", "New/"))
        .committed(Object::Task(task_in(3, "New", "")));
    assert_eq!(
        sent(&s),
        vec![(oid(3), Create), (oid(2), Create), (oid(1), Create)]
    );
}

#[test]
fn a_section_is_created_before_a_task_moves_into_it() {
    for (section, moved) in BOTH_OID_ORDERS {
        let s = held_upstream(&[task_in(moved, "m", "Home/")]);
        s.filed(task_in(section, "S", "Home/"))
            .committed(Object::Task(task_in(moved, "m", "Home/S/")));
        assert_eq!(sent(&s), vec![(oid(section), Create), (oid(moved), Update)]);
    }
}

fn refusing_removals(ms: &[RemoteMutation]) -> Vec<MutationOutcome> {
    ms.iter()
        .map(|m| MutationOutcome {
            oid: m.oid.clone(),
            verdict: match m.op {
                Delete => MutationVerdict::Refused {
                    why: "stopped".into(),
                    unasked: false,
                },
                _ => MutationVerdict::Accepted { remote_id: None },
            },
        })
        .collect()
}

#[test]
fn a_push_that_stopped_after_the_move_resends_only_the_removal() {
    for (parent, child) in BOTH_OID_ORDERS {
        let s = held_upstream(&[
            task_in(parent, "P", "Home/"),
            task_in(child, "c", "Home/P/"),
        ]);
        s.filed(task_in(child, "c", "Home/")).deleted(&oid(parent));
        let (l, _) = pushing(refusing_removals);
        s.dam().push(remotes_launched_by(&l), None).unwrap();
        assert_eq!(sent(&s), vec![(oid(parent), Delete)]);
    }
}

#[test]
fn a_retry_of_a_refused_push_resends_the_move_before_the_removal() {
    for (parent, child) in BOTH_OID_ORDERS {
        let s = held_upstream(&[
            task_in(parent, "P", "Home/"),
            task_in(child, "c", "Home/P/"),
        ]);
        s.filed(task_in(child, "c", "Home/")).deleted(&oid(parent));
        let (l, first) = pushing(refusing_all("temporary failure"));
        s.dam().push(remotes_launched_by(&l), None).unwrap();
        let first: Vec<_> = first
            .borrow()
            .iter()
            .map(|m| (m.oid.clone(), m.op))
            .collect();
        assert_eq!(first, vec![(oid(child), Update)]);
        assert_eq!(sent(&s), vec![(oid(child), Update), (oid(parent), Delete)]);
    }
}
