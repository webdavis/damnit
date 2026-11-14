use dam_domain::{Field, Object, Oid, Task};

use super::*;
use crate::remote::{MutationOp, RemoteMutation};

fn object(byte: u8) -> Oid {
    Oid::generate(&mut |b: &mut [u8]| b.fill(byte))
}

fn write(op: MutationOp, subject: &str, body: &str) -> RemoteMutation {
    let mut task = Task::new(object(1), subject);
    task.base.body = body.into();
    RemoteMutation {
        op,
        oid: object(1),
        idempotency_key: String::new(),
        remote_id: Some("r1".into()),
        container_remote_id: None,
        object: Some(Object::Task(task)),
        fields: vec![Field::Subject],
    }
}

#[test]
fn a_key_is_uuid_shaped_with_the_last_character_reserved() {
    let k = key(0, &write(MutationOp::Update, "a", ""));
    assert_eq!(k.len(), 36);
    let groups: Vec<&str> = k.split('-').collect();
    assert_eq!(
        groups.iter().map(|g| g.len()).collect::<Vec<_>>(),
        vec![8, 4, 4, 4, 12]
    );
    assert!(k.chars().all(|c| c == '-' || c.is_ascii_hexdigit()), "{k}");
    assert!(k.ends_with('0'), "the ordinal character is free: {k}");
}

#[test]
fn the_same_write_after_the_same_answers_yields_the_same_key() {
    let update = write(MutationOp::Update, "a", "");
    assert_eq!(key(3, &update), key(3, &update.clone()));
}

#[test]
fn another_answer_object_or_operation_yields_another_key() {
    let update = write(MutationOp::Update, "a", "");
    let elsewhere = RemoteMutation {
        oid: object(2),
        ..update.clone()
    };
    let delete = RemoteMutation {
        op: MutationOp::Delete,
        ..update.clone()
    };
    assert_ne!(key(0, &update), key(1, &update));
    assert_ne!(key(0, &update), key(0, &elsewhere));
    assert_ne!(key(0, &update), key(0, &delete));
}

#[test]
fn an_update_s_key_follows_the_declared_fields_and_their_values_only() {
    let update = write(MutationOp::Update, "a", "");
    let other_value = write(MutationOp::Update, "b", "");
    let undeclared_change = write(MutationOp::Update, "a", "notes");
    let more_fields = RemoteMutation {
        fields: vec![Field::Subject, Field::Body],
        ..update.clone()
    };
    assert_ne!(key(0, &update), key(0, &other_value));
    assert_ne!(key(0, &update), key(0, &more_fields));
    assert_eq!(key(0, &update), key(0, &undeclared_change));
}

#[test]
fn a_create_s_key_follows_the_object_and_its_answers_and_not_its_content() {
    let create = write(MutationOp::Create, "a", "");
    let edited = write(MutationOp::Create, "b", "notes");
    assert_eq!(key(0, &create), key(0, &edited));
    assert_ne!(key(0, &create), key(1, &edited));
}
