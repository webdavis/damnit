use dam_domain::{Author, Change, Channel, Item, Object, Path, Timestamp, Token, TokenAuthor};

use crate::testing::prelude::*;
use crate::testing::{Scenario, oid, task_in};
use crate::use_cases::prefix::Named;

fn removal_by_tool(s: &Scenario, before: Object) -> Item {
    let tool = TokenAuthor {
        name: "tool".into(),
        key_id: Token::generate(&mut |b: &mut [u8]| b.fill(3))
            .key_id()
            .clone(),
        channel: Channel::Local,
    };
    let change = Change::Delete { before };
    let item = Item::propose(tool, change, Timestamp::UNIX_EPOCH, &mut |b| b.fill(9));
    s.store.put_item(&item).unwrap();
    item
}

fn trip_with_a_child() -> (Scenario, Item) {
    let s = Scenario::default();
    let trip = Object::Task(task_in(1, "trip", "home"));
    s.committed(trip.clone());
    s.committed(Object::Task(task_in(2, "book flights", "home/trip")));
    let item = removal_by_tool(&s, trip);
    (s, item)
}

fn owners_staged_after(s: &Scenario) -> Object {
    let staged = s.store.staged().unwrap();
    let mine = staged
        .iter()
        .find(|a| a.author == Author::Owner && a.change.oid() == &oid(2))
        .unwrap();
    mine.change.after().unwrap().clone()
}

#[test]
fn a_childs_move_up_is_staged_as_the_owners_and_leaves_his_unstaged_edit_unstaged() {
    let (s, item) = trip_with_a_child();
    let mut edited = task_in(2, "book flights", "home/trip");
    edited.base.body = "window seat".into();
    s.working(Object::Task(edited));
    s.dam().add(&[Named::Item(item.hash)]).unwrap();
    let staged = owners_staged_after(&s);
    assert_eq!(staged.base().path, Path::parse("home").unwrap());
    assert_eq!(staged.base().body, "");
    let working = s.store.get(&oid(2)).unwrap().unwrap();
    assert_eq!(working.base().body, "window seat");
}

#[test]
fn a_childs_move_up_joins_the_edit_the_owner_already_staged() {
    let (s, item) = trip_with_a_child();
    s.working(Object::Task(task_in(2, "book the flights", "home/trip")));
    s.dam().add(&[Named::Object(oid(2))]).unwrap();
    s.dam().add(&[Named::Item(item.hash)]).unwrap();
    let staged = owners_staged_after(&s);
    assert_eq!(staged.base().path, Path::parse("home").unwrap());
    assert_eq!(staged.base().subject, "book the flights");
}
