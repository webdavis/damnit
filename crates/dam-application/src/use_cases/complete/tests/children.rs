use super::super::*;
use super::{at, filed, path_of, task_of};
use crate::testing::{Scenario, oid};
use dam_domain::{Blockers, Completion};

#[test]
fn a_root_task_with_objects_in_subfolders_has_no_children() {
    let s = Scenario::default().drawing_from(9);
    let milk = filed(&s, at(1, "buy oat milk", ""));
    filed(&s, at(2, "standup notes", "work"));
    filed(&s, at(3, "another", "work/clients/acme"));
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
    let offsite = filed(&s, at(1, "plan the offsite", "work"));
    filed(&s, at(2, "unrelated project note", "work/clients"));
    filed(&s, at(3, "another", "work/clients/acme"));
    assert!(s.dam().plan_complete(&offsite).unwrap().blockers.is_empty());
}

#[test]
fn open_children_and_dependencies_block_and_force_no_refuses() {
    let s = Scenario::default().drawing_from(9);
    let mut plan = at(1, "plan", "work");
    plan.base.depends.push(oid(3));
    let plan = filed(&s, plan);
    let step = filed(&s, at(2, "step", "work/plan"));
    let dep = filed(&s, at(3, "blocker", ""));
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
    let plan = filed(&s, at(1, "plan", "work"));
    let mut step = at(2, "step", "work/plan");
    step.completion = Completion::Done { at: None };
    filed(&s, step);
    assert!(s.dam().plan_complete(&plan).unwrap().blockers.is_empty());
}

#[test]
fn forcing_with_default_dispositions_completes_and_leaves_children_where_they_are() {
    let s = Scenario::default().drawing_from(9);
    let plan = filed(&s, at(1, "plan", "work"));
    let step = filed(&s, at(2, "step", "work/plan"));
    s.dam()
        .complete(&plan, Force::With(Dispositions::default()), Tracked::No)
        .unwrap();
    assert!(task_of(&s, &plan).completion.is_done());
    assert_eq!(path_of(&s, &step), "work/plan/");
}
