use dam_domain::{Authored, Change, Channel, Item, Object, Timestamp, Token, TokenAuthor};

use super::Reopened;
use crate::testing::prelude::*;
use crate::testing::{Scenario, oid, task_in};
use crate::use_cases::prefix::Named;

fn sent_by_tool(s: &Scenario, change: Change) -> Item {
    let tool = TokenAuthor {
        name: "tool".into(),
        key_id: Token::generate(&mut |b: &mut [u8]| b.fill(3))
            .key_id()
            .clone(),
        channel: Channel::Local,
    };
    let item = Item::propose(tool, change, Timestamp::UNIX_EPOCH, &mut |b| b.fill(9));
    s.store.put_item(&item).unwrap();
    item
}

fn trip() -> Object {
    Object::Task(task_in(1, "trip", "home"))
}

fn trip_with_a_child_and_a_dependent() -> Scenario {
    let s = Scenario::default();
    s.committed(trip());
    s.committed(Object::Task(task_in(2, "book flights", "home/trip")));
    let mut pack = task_in(3, "pack", "home");
    pack.base.depends = vec![oid(1)];
    s.committed(Object::Task(pack));
    s
}

fn layers(s: &Scenario) -> (Vec<Object>, Vec<Authored>) {
    (s.store.all().unwrap(), s.store.staged().unwrap())
}

fn added_then_reset(s: &Scenario, change: Change) -> Reopened {
    let item = sent_by_tool(s, change);
    s.dam().add(&[Named::Item(item.hash.clone())]).unwrap();
    s.dam().reopen(&item.hash).unwrap()
}

#[test]
fn reset_of_an_added_removal_sets_back_the_child_and_the_dependent() {
    let s = trip_with_a_child_and_a_dependent();
    let before = layers(&s);

    let reopened = added_then_reset(&s, Change::Delete { before: trip() });

    assert_eq!(layers(&s), before);
    assert_eq!(
        reopened,
        Reopened::Unstaged {
            oid: oid(1),
            kept: Vec::new()
        }
    );
}

#[test]
fn reset_of_an_added_removal_puts_back_the_owners_staged_edit_of_a_child() {
    let s = trip_with_a_child_and_a_dependent();
    s.working(Object::Task(task_in(2, "book the flights", "home/trip")));
    s.dam().add(&[Named::Object(oid(2))]).unwrap();
    let before = layers(&s);

    added_then_reset(&s, Change::Delete { before: trip() });

    assert_eq!(layers(&s), before);
}

#[test]
fn reset_of_an_added_rename_moves_the_child_back() {
    let s = trip_with_a_child_and_a_dependent();
    let before = layers(&s);

    added_then_reset(
        &s,
        Change::Update {
            before: trip(),
            after: Object::Task(task_in(1, "journey", "home")),
        },
    );

    assert_eq!(layers(&s), before);
}

#[test]
fn reset_keeps_and_names_a_task_the_owner_changed_after_the_add() {
    let s = trip_with_a_child_and_a_dependent();
    let item = sent_by_tool(&s, Change::Delete { before: trip() });
    s.dam().add(&[Named::Item(item.hash.clone())]).unwrap();
    let mut edited = s.store.get(&oid(3)).unwrap().unwrap();
    edited.base_mut().body = "light".into();
    s.working(edited.clone());

    let reopened = s.dam().reopen(&item.hash).unwrap();

    assert_eq!(
        reopened,
        Reopened::Unstaged {
            oid: oid(1),
            kept: vec![oid(3)]
        }
    );
    assert_eq!(s.store.get(&oid(3)).unwrap(), Some(edited));
    let child = s.store.get(&oid(2)).unwrap().unwrap();
    assert_eq!(child.base().path.to_string(), "home/trip/");
}
