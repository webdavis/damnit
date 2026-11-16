use super::*;
use dam_application::{MutationOp, RemoteMutation};
use dam_domain::{Focus, Object, Oid, Task};
use jiff::civil::date;

fn focused_task() -> Object {
    let mut t = Task::new(Oid::generate(&mut |x: &mut [u8]| x.fill(1)), "taxes");
    t.estimate = Some(30);
    t.focus = Some(Focus {
        day: date(2026, 11, 12),
        set_by: "owner".into(),
        set_at: "2026-11-12T08:00:00Z".parse().unwrap(),
    });
    Object::Task(t)
}

#[test]
fn the_store_keeps_a_focus_mark_and_an_estimate() {
    let object = focused_task();
    assert_eq!(from_wire(&to_wire(&object, None)).unwrap(), object);
}

#[test]
fn a_focus_mark_never_reaches_a_remote() {
    let object = focused_task();
    let sent = mutation_to_wire(RemoteMutation {
        op: MutationOp::Create,
        oid: object.oid().clone(),
        idempotency_key: "k".into(),
        remote_id: None,
        container_remote_id: None,
        object: Some(object),
        fields: vec![Field::Subject],
    });
    let task = sent.object.unwrap().task.unwrap();
    assert_eq!(task.focus, None);
    assert_eq!(task.estimate, Some(30));
}

#[test]
fn a_task_names_its_event_attached_and_still_reads_a_row_that_named_it_event() {
    let event = Oid::generate(&mut |x: &mut [u8]| x.fill(2));
    let mut t = Task::new(Oid::generate(&mut |x: &mut [u8]| x.fill(1)), "prep");
    t.event = Some(event.clone());
    let object = Object::Task(t);
    let row = serde_json::to_value(to_wire(&object, None)).unwrap();
    assert_eq!(row["task"]["attached"], event.to_string());
    assert!(row["task"].get("event").is_none());
    let mut old = row.clone();
    old["task"]["event"] = old["task"]["attached"].take();
    old["task"].as_object_mut().unwrap().remove("attached");
    let old: dam_protocol::WireObject = serde_json::from_value(old).unwrap();
    assert_eq!(from_wire(&old).unwrap(), object);
}

#[test]
fn a_mutation_carries_the_container_it_names() {
    let object = focused_task();
    let sent = mutation_to_wire(RemoteMutation {
        op: MutationOp::Update,
        oid: object.oid().clone(),
        idempotency_key: "k".into(),
        remote_id: Some("i:c".into()),
        container_remote_id: Some("i:r2".into()),
        object: Some(object),
        fields: vec![Field::Path],
    });
    assert_eq!(sent.container_remote_id.as_deref(), Some("i:r2"));
}

fn oid(b: u8) -> Oid {
    Oid::generate(&mut |x: &mut [u8]| x.fill(b))
}

fn task_inside(parent: Option<Oid>) -> Object {
    let mut t = Task::new(oid(1), "milk");
    t.base.parent = parent;
    Object::Task(t)
}

#[test]
fn the_stored_form_keeps_the_parent_under_its_own_key() {
    let object = task_inside(Some(oid(2)));
    let row = serde_json::to_value(to_wire(&object, None)).unwrap();
    assert_eq!(row["parent"], oid(2).to_string());
    assert_eq!(from_wire(&to_wire(&object, None)).unwrap(), object);
}

#[test]
fn an_object_with_no_parent_leaves_the_key_out_and_reads_back_as_none() {
    let row = serde_json::to_value(to_wire(&task_inside(None), None)).unwrap();
    assert!(row.get("parent").is_none());
    let read: WireObject = serde_json::from_value(row).unwrap();
    assert_eq!(from_wire(&read).unwrap(), task_inside(None));
}

#[test]
fn a_parent_that_is_not_an_oid_is_refused() {
    let mut wire = to_wire(&task_inside(None), None);
    wire.parent = Some("nope".into());
    assert!(from_wire(&wire).is_err());
}

#[test]
fn a_parent_never_reaches_a_remote() {
    let object = task_inside(Some(oid(2)));
    let sent = mutation_to_wire(RemoteMutation {
        op: MutationOp::Create,
        oid: object.oid().clone(),
        idempotency_key: "k".into(),
        remote_id: None,
        container_remote_id: None,
        object: Some(object),
        fields: vec![Field::Subject],
    });
    assert_eq!(sent.object.unwrap().parent, None);
}
