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
        object: Some(object),
        fields: vec![Field::Subject],
    });
    let task = sent.object.unwrap().task.unwrap();
    assert_eq!(task.focus, None);
    assert_eq!(task.estimate, Some(30));
}
