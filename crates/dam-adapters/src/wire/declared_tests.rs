use dam_application::{MutationOp, RemoteMutation};
use dam_domain::{Field, Object, Oid, Priority, Task};

use super::declared_on_the_wire;

fn update(subject: &str, body: &str, fields: Vec<Field>) -> RemoteMutation {
    let oid = Oid::generate(&mut |b: &mut [u8]| b.fill(1));
    let mut task = Task::new(oid.clone(), subject);
    task.base.body = body.into();
    task.priority = Priority::HIGHEST;
    RemoteMutation {
        op: MutationOp::Update,
        oid,
        idempotency_key: String::new(),
        remote_id: Some("r1".into()),
        container_remote_id: None,
        object: Some(Object::Task(task)),
        fields,
    }
}

#[test]
fn the_same_declared_values_put_the_same_form_on_the_wire() {
    let first = update("a", "", vec![Field::Subject, Field::Priority]);
    let reordered = update("a", "", vec![Field::Priority, Field::Subject]);
    assert_eq!(
        declared_on_the_wire(&first),
        declared_on_the_wire(&reordered)
    );
}

#[test]
fn another_declared_value_puts_another_form_on_the_wire() {
    let first = update("a", "", vec![Field::Subject]);
    let renamed = update("b", "", vec![Field::Subject]);
    assert_ne!(declared_on_the_wire(&first), declared_on_the_wire(&renamed));
}

#[test]
fn a_field_that_is_not_declared_leaves_the_form_alone() {
    let first = update("a", "", vec![Field::Subject]);
    let other_body = update("a", "notes", vec![Field::Subject]);
    assert_eq!(
        declared_on_the_wire(&first),
        declared_on_the_wire(&other_body)
    );
}

#[test]
fn the_form_is_each_declared_field_in_field_order_with_its_wire_value() {
    let declared = update(
        "a",
        "notes",
        vec![Field::Priority, Field::Done, Field::Body, Field::Focus],
    );
    assert_eq!(
        declared_on_the_wire(&declared),
        r#"[["body","notes"],["priority",1],["done",false],["focus",null]]"#
    );
}
