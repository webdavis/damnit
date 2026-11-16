use dam_application::{Declared, MutationOp, RemoteMutation};
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
        container_oid: None,
        object: Some(Object::Task(task)),
        fields,
    }
}

fn declared(mutation: &RemoteMutation) -> Declared {
    declared_on_the_wire(mutation).unwrap()
}

#[test]
fn each_declared_field_carries_its_wire_value_in_field_order() {
    let mutation = update(
        "a",
        "notes",
        vec![Field::Priority, Field::Done, Field::Body, Field::Focus],
    );
    let declared = declared(&mutation);
    let in_order: Vec<(&str, &str)> = declared
        .iter()
        .map(|(f, v)| (f.as_str(), v.as_str()))
        .collect();
    assert_eq!(
        in_order,
        vec![
            ("body", r#""notes""#),
            ("priority", "1"),
            ("done", "false"),
            ("focus", "null"),
        ]
    );
}

#[test]
fn another_value_of_a_declared_field_puts_another_value_on_the_wire() {
    let first = update("a", "", vec![Field::Subject]);
    let renamed = update("b", "", vec![Field::Subject]);
    assert_ne!(declared(&first), declared(&renamed));
}

#[test]
fn a_field_that_is_not_declared_is_left_out() {
    let first = update("a", "", vec![Field::Subject]);
    let other_body = update("a", "notes", vec![Field::Subject]);
    assert_eq!(declared(&first), declared(&other_body));
    assert!(!declared(&first).contains_key(&Field::Body));
}

#[test]
fn a_write_with_no_object_declares_nothing() {
    let delete = RemoteMutation {
        op: MutationOp::Delete,
        object: None,
        fields: vec![],
        ..update("a", "", vec![])
    };
    assert!(declared(&delete).is_empty());
}
