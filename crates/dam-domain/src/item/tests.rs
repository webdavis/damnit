use super::*;
use crate::{Channel, Object, Task, Token};

fn tool() -> TokenAuthor {
    TokenAuthor {
        name: "tool".into(),
        key_id: Token::generate(&mut |b: &mut [u8]| b.fill(1))
            .key_id()
            .clone(),
        channel: Channel::Local,
    }
}

fn task(byte: u8, subject: &str) -> Object {
    Object::Task(Task::new(Oid::generate(&mut |b| b.fill(byte)), subject))
}

#[test]
fn a_new_items_hash_is_the_oid_its_task_will_have() {
    let after = task(4, "call the dentist");
    let item = Item::propose(
        tool(),
        Change::Create {
            after: after.clone(),
        },
        Timestamp::UNIX_EPOCH,
        &mut |b| b.fill(9),
    );
    assert_eq!(item.hash.oid(), after.oid());
    assert_eq!(item.kind(), ItemKind::New);
    assert_eq!(item.target(), after.oid());
    assert!(item.is_waiting());
}

#[test]
fn a_change_item_draws_a_hash_of_its_own_and_targets_the_task() {
    let before = task(4, "call the dentist");
    let after = task(4, "call the dentist today");
    let item = Item::propose(
        tool(),
        Change::Update { before, after },
        Timestamp::UNIX_EPOCH,
        &mut |b| b.fill(9),
    );
    assert_eq!(item.hash.oid(), &Oid::generate(&mut |b| b.fill(9)));
    assert_eq!(item.target(), &Oid::generate(&mut |b| b.fill(4)));
    assert_eq!(item.kind(), ItemKind::Change);
}

#[test]
fn an_outcome_other_than_waiting_carries_its_time() {
    let at = Timestamp::UNIX_EPOCH;
    for outcome in [
        Outcome::Waiting,
        Outcome::Added(at),
        Outcome::Declined(at),
        Outcome::Withdrawn(at),
    ] {
        assert_eq!(
            Outcome::from_parts(outcome.as_str(), outcome.at()),
            Some(outcome)
        );
    }
    assert_eq!(Outcome::from_parts("added", None), None);
    assert_eq!(Outcome::from_parts("waiting", Some(at)), None);
}
