use dam_domain::{Field, Object, Oid, Task};

use super::*;
use crate::ports::Declared;
use crate::remote::{MutationOp, RemoteMutation};

fn object(byte: u8) -> Oid {
    Oid::generate(&mut |b: &mut [u8]| b.fill(byte))
}

fn write(op: MutationOp) -> RemoteMutation {
    RemoteMutation {
        op,
        oid: object(1),
        idempotency_key: String::new(),
        remote_id: Some("r1".into()),
        container_remote_id: None,
        object: Some(Object::Task(Task::new(object(1), "a"))),
        fields: vec![Field::Subject],
    }
}

fn subject(value: &str) -> Declared {
    Declared::from([(Field::Subject, value.to_string())])
}

#[test]
fn a_key_is_uuid_shaped_with_the_last_character_reserved() {
    let k = key(0, 0, &write(MutationOp::Update), &subject("a"));
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
fn the_same_write_at_the_same_count_and_position_yields_the_same_key() {
    let update = write(MutationOp::Update);
    assert_eq!(
        key(3, 1, &update, &subject("a")),
        key(3, 1, &update.clone(), &subject("a"))
    );
}

#[test]
fn an_update_s_key_follows_its_count_position_content_and_container() {
    let update = write(MutationOp::Update);
    let refiled = RemoteMutation {
        container_remote_id: Some("p1".into()),
        ..update.clone()
    };
    let first = key(0, 0, &update, &subject("a"));
    assert_ne!(first, key(1, 0, &update, &subject("a")), "count");
    assert_ne!(first, key(0, 2, &update, &subject("a")), "position");
    assert_ne!(first, key(0, 0, &update, &subject("b")), "content");
    assert_ne!(first, key(0, 0, &refiled, &subject("a")), "container");
}

#[test]
fn a_delete_s_key_follows_its_count_and_position() {
    let delete = write(MutationOp::Delete);
    let none = Declared::new();
    let first = key(0, 0, &delete, &none);
    assert_ne!(first, key(1, 0, &delete, &none));
    assert_ne!(first, key(0, 1, &delete, &none));
    assert_ne!(first, key(0, 0, &write(MutationOp::Update), &none));
}

#[test]
fn a_create_s_key_follows_its_object_and_count_only() {
    let create = write(MutationOp::Create);
    let elsewhere = RemoteMutation {
        oid: object(2),
        ..create.clone()
    };
    let first = key(0, 0, &create, &subject("a"));
    assert_eq!(first, key(0, 3, &create, &subject("b")));
    assert_ne!(first, key(1, 0, &create, &subject("a")));
    assert_ne!(first, key(0, 0, &elsewhere, &subject("a")));
}
