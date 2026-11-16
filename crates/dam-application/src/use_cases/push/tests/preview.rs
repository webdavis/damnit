use crate::ports::Declared;
use crate::remote::MutationOp;
use crate::testing::prelude::*;
use crate::testing::{Scenario, oid, task, task_caps, todoist};

fn on_the_wire(_: &crate::remote::RemoteMutation) -> Result<Declared, crate::ports::HelperError> {
    Ok(Declared::new())
}

fn contacted(s: &Scenario) {
    s.store
        .set_accepted_kinds(&todoist(), &task_caps().kinds)
        .unwrap();
    s.store
        .set_accepted_fields(&todoist(), &task_caps().fields)
        .unwrap();
}

#[test]
fn a_preview_plans_what_a_push_would_send_and_writes_nothing() {
    let s = Scenario::default();
    s.committed(task(1));
    contacted(&s);
    let before = s.store.clone();
    let previews = s.dam().push_preview(None, &on_the_wire).unwrap();
    let plan = previews[0].plan.as_ref().unwrap();
    assert_eq!(plan.mutations.len(), 1);
    assert_eq!(plan.mutations[0].op, MutationOp::Create);
    assert_eq!(plan.mutations[0].oid, oid(1));
    assert_eq!(s.store, before);
}

#[test]
fn a_remote_never_contacted_has_no_plan() {
    let s = Scenario::default();
    s.committed(task(1));
    let before = s.store.clone();
    let previews = s.dam().push_preview(None, &on_the_wire).unwrap();
    assert_eq!(previews[0].remote, todoist());
    assert!(previews[0].plan.is_none());
    assert_eq!(s.store, before);
}
