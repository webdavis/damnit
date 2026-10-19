use super::*;
use dam_application::{Notice, RemoteName};
use dam_domain::{Object, Oid, Task};

fn oid(b: u8) -> Oid {
    Oid::generate(&mut |x: &mut [u8]| x.fill(b))
}

#[test]
fn an_object_round_trips_through_json() {
    let o = Object::Task(Task::new(oid(1), "milk"));
    assert_eq!(object_from_json(&object_to_json(&o).unwrap()).unwrap(), o);
}

#[test]
fn ops_round_trip_and_bad_text_is_an_error() {
    for op in [Op::Create, Op::Update, Op::Delete] {
        assert_eq!(op_from_text(op_to_text(op)).unwrap(), op);
    }
    assert!(op_from_text("bogus").is_err());
}

#[test]
fn a_stored_change_reads_back_only_when_its_op_agrees_with_its_sides() {
    let milk = object_to_json(&Object::Task(Task::new(oid(1), "milk"))).unwrap();
    let oat = object_to_json(&Object::Task(Task::new(oid(1), "oat milk"))).unwrap();
    assert_eq!(
        change_from_row("update", Some(milk.clone()), Some(oat.clone()))
            .unwrap()
            .op(),
        Op::Update
    );
    for (op, before, after) in [
        ("update", None, Some(oat.clone())),
        ("create", Some(milk.clone()), Some(oat)),
        ("update", Some(milk.clone()), Some(milk.clone())),
        ("delete", None, Some(milk)),
        ("create", None, None),
    ] {
        assert_eq!(
            change_from_row(op, before, after),
            Err(StoreError::Failed(format!(
                "stored {op} change disagrees with its before and after"
            )))
        );
    }
}

#[test]
fn every_notice_variant_round_trips() {
    let notices = vec![
        Notice::RemovedUpstream {
            remote: RemoteName("t".into()),
            oid: oid(1),
            subject: "s".into(),
        },
        Notice::EventCancelled {
            oid: oid(2),
            subject: "e".into(),
            attached: 3,
        },
        Notice::PushFailed {
            remote: RemoteName("t".into()),
            oid: oid(3),
            why: "w".into(),
        },
        Notice::PullFailed {
            remote: RemoteName("t".into()),
            why: "w".into(),
        },
        Notice::KindChanged {
            oid: oid(4),
            ours: Kind::Task,
            theirs: Kind::Event,
        },
    ];
    for n in notices {
        assert_eq!(notice_from_json(&notice_to_json(&n).unwrap()).unwrap(), n);
    }
}
