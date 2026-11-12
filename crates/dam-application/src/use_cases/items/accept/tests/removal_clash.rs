use dam_domain::{Author, Change, Channel, Field, Item, Object, Timestamp, Token, TokenAuthor};

use crate::testing::prelude::*;
use crate::testing::{Scenario, oid, task_in};
use crate::use_cases::items::refusal::ItemRefusal;
use crate::use_cases::prefix::Named;

fn removal_by_vpt(s: &Scenario, before: Object) -> Item {
    let vpt = TokenAuthor {
        name: "vpt".into(),
        key_id: Token::generate(&mut |b: &mut [u8]| b.fill(3))
            .key_id()
            .clone(),
        channel: Channel::Local,
    };
    let change = Change::Delete { before };
    let item = Item::propose(vpt, change, Timestamp::UNIX_EPOCH, &mut |b| b.fill(9));
    s.store.put_item(&item).unwrap();
    item
}

#[test]
fn adding_a_removal_of_a_task_the_owner_edited_and_has_not_saved_stops_the_add() {
    let s = Scenario::default();
    let rent = Object::Task(task_in(1, "pay rent", "home"));
    s.committed(rent.clone());
    let item = removal_by_vpt(&s, rent);
    let edited = Object::Task(task_in(1, "pay rent by friday", "home"));
    s.working(edited.clone());

    assert_eq!(
        s.dam().add(&[Named::Item(item.hash.clone())]),
        Err(ItemRefusal::Clash {
            hash: item.hash.clone(),
            target: oid(1),
            fields: vec![Field::Subject],
        }
        .into())
    );
    assert_eq!(s.store.get(&oid(1)).unwrap(), Some(edited));
    assert!(s.store.item(&item.hash).unwrap().unwrap().is_waiting());
    let staged = s.dam().add_all().unwrap();
    assert!(
        staged.iter().all(|a| a.author == Author::Owner),
        "{staged:?}"
    );
    assert!(s.store.item(&item.hash).unwrap().unwrap().is_waiting());
}
