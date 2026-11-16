mod visibility;

use dam_domain::{Change, Channel, Item, ItemHash, Object, Outcome, Timestamp, Token, TokenAuthor};

use crate::testing::prelude::{ItemRepository, ObjectRepository};
use crate::testing::{Scenario, oid, task, task_in};
use crate::use_cases::items::refusal::ItemRefusal;
use crate::use_cases::prefix::Named;

fn tool() -> TokenAuthor {
    TokenAuthor {
        name: "tool".into(),
        key_id: Token::generate(&mut |b: &mut [u8]| b.fill(1))
            .key_id()
            .clone(),
        channel: Channel::Local,
    }
}

fn set() -> ItemHash {
    ItemHash::new(oid(90))
}

fn waiting(s: &Scenario, byte: u8, change: Change) -> ItemHash {
    let item =
        Item::propose(tool(), change, Timestamp::UNIX_EPOCH, &mut |b| b.fill(byte)).in_set(set());
    s.store.put_item(&item).unwrap();
    item.hash
}

fn a_new_task(s: &Scenario, byte: u8) -> ItemHash {
    waiting(s, byte, Change::Create { after: task(byte) })
}

fn a_change_to_an_uncommitted_task(s: &Scenario, byte: u8) -> ItemHash {
    let before = Object::Task(task_in(byte, "old", "work"));
    s.working(before.clone());
    let after = Object::Task(task_in(byte, "new", "work"));
    waiting(s, byte, Change::Update { before, after })
}

fn outcome(s: &Scenario, hash: &ItemHash) -> Outcome {
    s.store.item(hash).unwrap().unwrap().outcome
}

#[test]
fn adding_the_set_takes_every_item_in_it() {
    let s = Scenario::default();
    let (one, two) = (a_new_task(&s, 1), a_new_task(&s, 2));
    s.dam().add(&[Named::Set(set())]).unwrap();
    assert!(matches!(outcome(&s, &one), Outcome::Added(_)));
    assert!(matches!(outcome(&s, &two), Outcome::Added(_)));
    assert!(s.store.get(&oid(2)).unwrap().is_some());
}

#[test]
fn a_set_with_one_item_that_no_longer_applies_takes_nothing() {
    let s = Scenario::default();
    let fine = a_new_task(&s, 1);
    a_change_to_an_uncommitted_task(&s, 2);
    assert!(s.dam().add(&[Named::Set(set())]).is_err());
    assert_eq!(outcome(&s, &fine), Outcome::Waiting);
    assert!(s.store.get(&oid(1)).unwrap().is_none());
}

#[test]
fn add_all_takes_a_whole_set_or_leaves_it_whole() {
    let s = Scenario::default();
    let fine = a_new_task(&s, 1);
    a_change_to_an_uncommitted_task(&s, 2);
    s.dam().add_all().unwrap();
    assert_eq!(outcome(&s, &fine), Outcome::Waiting);
}

#[test]
fn an_item_in_a_set_cannot_be_taken_or_declined_alone() {
    let s = Scenario::default();
    let one = a_new_task(&s, 1);
    a_new_task(&s, 2);
    let in_a_set = Err(ItemRefusal::InASet(set()).into());
    assert_eq!(
        s.dam().add(&[Named::Item(one.clone())]).map(|_| ()),
        in_a_set
    );
    assert_eq!(s.dam().dismiss(&one).map(|_| ()), in_a_set);
}

#[test]
fn removing_the_set_declines_every_item_in_it() {
    let s = Scenario::default();
    let (one, two) = (a_new_task(&s, 1), a_new_task(&s, 2));
    let declined = s.dam().dismiss_set(&set()).unwrap();
    assert_eq!(declined.len(), 2);
    assert!(matches!(outcome(&s, &one), Outcome::Declined(_)));
    assert!(matches!(outcome(&s, &two), Outcome::Declined(_)));
}

#[test]
fn a_set_is_named_by_its_hash_or_a_prefix() {
    let s = Scenario::default();
    a_new_task(&s, 1);
    let dam = s.dam();
    assert_eq!(dam.resolve_named(set().as_str()), Ok(Named::Set(set())));
    assert_eq!(
        dam.resolve_named(&set().as_str()[..6]),
        Ok(Named::Set(set()))
    );
}

#[test]
fn a_preview_of_adding_a_set_takes_nothing() {
    let s = Scenario::default();
    let one = a_new_task(&s, 1);
    let rehearsed = s
        .dam()
        .rehearse(|dam| dam.add(&[Named::Set(set())]))
        .unwrap();
    assert_eq!(rehearsed.ran.staged.len(), 1);
    assert_eq!(outcome(&s, &one), Outcome::Waiting);
    assert!(s.store.get(&oid(1)).unwrap().is_none());
}
