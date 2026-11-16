use dam_domain::Object;

use super::except::excepting_kitchen;
use super::parents::held_by;
use crate::ports::{AttemptRepository, Declared};
use crate::remote::MutationOp::{Create, Delete, Update};
use crate::testing::prelude::*;
use crate::testing::{Scenario, oid, task_caps, task_in, todoist};
use crate::use_cases::push::Skip;

fn on_the_wire(_: &crate::remote::RemoteMutation) -> Result<Declared, crate::ports::HelperError> {
    Ok(Declared::new())
}

fn every_kind_of_row() -> Scenario {
    let s = excepting_kitchen(Scenario::default());
    s.committed(Object::Task(task_in(1, "draft", "Home/")))
        .tracked_as(&oid(1), "r1")
        .committed(Object::Task(task_in(3, "old", "Home/")))
        .tracked_as(&oid(3), "r3")
        .pushed();
    s.committed(Object::Task(task_in(1, "final", "Home/")))
        .deleted(&oid(3))
        .committed(Object::Task(task_in(4, "new", "Home/")))
        .committed(Object::Task(task_in(5, "sink", "kitchen/")))
        .committed(Object::Task(task_in(7, "P", "kitchen/")))
        .committed(Object::Task(held_by(6, "c", "Home/", 7)));
    s.store
        .set_accepted_kinds(&todoist(), &task_caps().kinds)
        .unwrap();
    s.store
        .set_accepted_fields(&todoist(), &task_caps().fields)
        .unwrap();
    s
}

#[test]
fn a_preview_of_every_kind_of_row_leaves_the_store_as_it_was() {
    let s = every_kind_of_row();
    let before = s.store.clone();
    let previews = s.dam().push_preview(None, &on_the_wire).unwrap();
    let plan = previews[0].plan.as_ref().unwrap();
    let ops: Vec<_> = plan
        .mutations
        .iter()
        .map(|m| (m.oid.clone(), m.op))
        .collect();
    assert!(ops.contains(&(oid(1), Update)), "{ops:?}");
    assert!(ops.contains(&(oid(3), Delete)), "{ops:?}");
    let removed = plan.mutations.iter().find(|m| m.op == Delete).unwrap();
    assert_eq!(removed.object.as_ref().unwrap().base().subject, "old");
    assert!(ops.contains(&(oid(4), Create)), "{ops:?}");
    assert_eq!(
        plan.held.iter().map(|h| &h.oid).collect::<Vec<_>>(),
        [&oid(6)]
    );
    assert!(plan.held[0].why.contains("remote.todoist.except"));
    assert!(plan.skipped.contains(&(oid(5), Skip::Excepted)));
    assert_eq!(s.store, before);
    assert!(s.store.attempts(&todoist()).unwrap().is_empty());
    assert!(s.store.notices().unwrap().is_empty());
    assert_eq!(s.store.remote_id(&todoist(), &oid(4)).unwrap(), None);
}
