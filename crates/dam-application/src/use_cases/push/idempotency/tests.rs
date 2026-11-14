use dam_domain::{Field, Object, Oid, Task};

use super::*;
use crate::remote::{MutationOp, RemoteMutation};

fn object(byte: u8) -> Oid {
    Oid::generate(&mut |b: &mut [u8]| b.fill(byte))
}

fn write(op: MutationOp) -> RemoteMutation {
    let task = Task::new(object(1), "a");
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

const SUBJECT_A: &str = r#"[["subject","a"]]"#;
const SUBJECT_B: &str = r#"[["subject","b"]]"#;

#[test]
fn a_key_is_uuid_shaped_with_the_last_character_reserved() {
    let k = key(0, &write(MutationOp::Update), SUBJECT_A);
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
fn an_update_s_key_for_a_given_wire_form_is_pinned() {
    assert_eq!(
        key(2, &write(MutationOp::Update), SUBJECT_A),
        "09e4b74f-4a22-803e-827e-b97a627f30b0"
    );
}

#[test]
fn the_same_write_after_the_same_answers_yields_the_same_key() {
    let update = write(MutationOp::Update);
    assert_eq!(
        key(3, &update, SUBJECT_A),
        key(3, &update.clone(), SUBJECT_A)
    );
}

#[test]
fn another_answer_object_or_operation_yields_another_key() {
    let update = write(MutationOp::Update);
    let elsewhere = RemoteMutation {
        oid: object(2),
        ..update.clone()
    };
    let delete = RemoteMutation {
        op: MutationOp::Delete,
        ..update.clone()
    };
    assert_ne!(key(0, &update, SUBJECT_A), key(1, &update, SUBJECT_A));
    assert_ne!(key(0, &update, SUBJECT_A), key(0, &elsewhere, SUBJECT_A));
    assert_ne!(key(0, &update, SUBJECT_A), key(0, &delete, SUBJECT_A));
}

#[test]
fn an_update_s_key_follows_what_it_puts_on_the_wire_and_its_container() {
    let update = write(MutationOp::Update);
    let refiled = RemoteMutation {
        container_remote_id: Some("p1".into()),
        ..update.clone()
    };
    let more_fields = RemoteMutation {
        fields: vec![Field::Subject, Field::Body],
        ..update.clone()
    };
    assert_ne!(key(0, &update, SUBJECT_A), key(0, &update, SUBJECT_B));
    assert_ne!(key(0, &update, SUBJECT_A), key(0, &refiled, SUBJECT_A));
    assert_eq!(key(0, &update, SUBJECT_A), key(0, &more_fields, SUBJECT_A));
}

#[test]
fn a_create_s_key_follows_the_object_and_its_answers_and_not_its_content() {
    let create = write(MutationOp::Create);
    assert_eq!(key(0, &create, SUBJECT_A), key(0, &create, SUBJECT_B));
    assert_ne!(key(0, &create, SUBJECT_A), key(1, &create, SUBJECT_A));
}
