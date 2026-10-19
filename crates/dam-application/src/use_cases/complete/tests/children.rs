use super::super::*;
use super::task_of;
use crate::testing::{Scenario, oid, task_in};
use dam_domain::{Blockers, Completion};

#[test]
fn a_root_task_with_objects_in_subfolders_has_no_children() {
    let s = Scenario::default().drawing_from(9);
    let milk = oid(1);
    s.filed(task_in(1, "buy oat milk", ""));
    s.filed(task_in(2, "standup notes", "work"));
    s.filed(task_in(3, "another", "work/clients/acme"));
    let dam = s.dam();
    assert!(dam.plan_complete(&milk).unwrap().blockers.is_empty());
    assert_eq!(
        dam.complete(&milk, Force::No, Tracked::No).unwrap(),
        Completed::Done
    );
}

#[test]
fn objects_in_a_siblings_folder_are_not_children() {
    let s = Scenario::default();
    let offsite = oid(1);
    s.filed(task_in(1, "plan the offsite", "work"));
    s.filed(task_in(2, "unrelated project note", "work/clients"));
    s.filed(task_in(3, "another", "work/clients/acme"));
    assert!(s.dam().plan_complete(&offsite).unwrap().blockers.is_empty());
}

#[test]
fn open_children_and_dependencies_block_and_force_no_refuses() {
    let s = Scenario::default().drawing_from(9);
    let mut plan = task_in(1, "plan", "work");
    plan.base.depends.push(oid(3));
    s.filed(plan)
        .filed(task_in(2, "step", "work/plan"))
        .filed(task_in(3, "blocker", ""));
    let (plan, step, dep) = (oid(1), oid(2), oid(3));
    let dam = s.dam();
    let found = dam.plan_complete(&plan).unwrap();
    assert_eq!(
        found.blockers,
        Blockers {
            dependencies: vec![dep.clone()],
            children: vec![step.clone()],
        }
    );
    let err = dam.complete(&plan, Force::No, Tracked::No).unwrap_err();
    assert!(matches!(
        err,
        UseCaseError::Refused(Refusal::Blocked { .. })
    ));
    assert!(!task_of(&s, &plan).completion.is_done());
}

#[test]
fn done_children_do_not_block() {
    let s = Scenario::default();
    s.filed(task_in(1, "plan", "work"));
    let plan = oid(1);
    let mut step = task_in(2, "step", "work/plan");
    step.completion = Completion::Done { at: None };
    s.filed(step);
    assert!(s.dam().plan_complete(&plan).unwrap().blockers.is_empty());
}

#[test]
fn forcing_with_default_dispositions_completes_and_leaves_children_where_they_are() {
    let s = Scenario::default().drawing_from(9);
    s.filed(task_in(1, "plan", "work"))
        .filed(task_in(2, "step", "work/plan"));
    let (plan, step) = (oid(1), oid(2));
    s.dam()
        .complete(&plan, Force::With(Dispositions::default()), Tracked::No)
        .unwrap();
    assert!(task_of(&s, &plan).completion.is_done());
    assert_eq!(s.path_of(&step), "work/plan/");
}
