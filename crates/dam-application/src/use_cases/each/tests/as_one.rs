use dam_domain::Permission::{TaskChange, TaskRead, TaskRemove};

use super::*;
use crate::service::Dam;
use crate::testing::prelude::ItemRepository;
use crate::testing::{allowed, holder};

fn token() -> crate::access::Holder {
    holder(vec![allowed(
        &[TaskRead, TaskChange, TaskRemove],
        Some(&["work", "home"]),
    )])
}

fn as_one() -> SetRun {
    SetRun {
        as_one: true,
        ..run()
    }
}

#[test]
fn as_one_puts_every_item_a_token_sends_in_one_new_set() {
    let s = work_and_home();
    let token = token();
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    dam.edit_each(&query("kind:task"), retitled("x"), as_one())
        .unwrap();
    let items = s.store.items().unwrap();
    assert_eq!(items.len(), 3);
    let set = items[0].set.clone().expect("the items wait as one set");
    assert!(items.iter().all(|item| item.set.as_ref() == Some(&set)));
    assert!(items.iter().all(|item| item.hash != set));
}

#[test]
fn without_as_one_each_item_waits_alone() {
    let s = work_and_home();
    let token = token();
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    dam.remove_each(&query("kind:task"), run()).unwrap();
    assert!(s.store.items().unwrap().iter().all(|i| i.set.is_none()));
}

#[test]
fn two_as_one_runs_make_two_sets() {
    let s = work_and_home();
    let token = token();
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    dam.edit_each(&query("path:work/"), retitled("x"), as_one())
        .unwrap();
    dam.edit_each(&query("path:home/"), retitled("y"), as_one())
        .unwrap();
    let items = s.store.items().unwrap();
    assert_ne!(items[0].set, items[2].set);
}

#[test]
fn a_preview_as_one_sends_nothing() {
    let s = work_and_home();
    let token = token();
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    let previewing = SetRun {
        preview: true,
        ..as_one()
    };
    dam.edit_each(&query("kind:task"), retitled("x"), previewing)
        .unwrap();
    assert!(s.store.items().unwrap().is_empty());
}

#[test]
fn a_set_never_takes_the_hash_of_an_item_already_kept() {
    use dam_domain::{Change, Item, Timestamp};
    let s = work_and_home().drawing_from(10);
    let token = token();
    let author = Dam {
        access: &token,
        ..s.dam()
    }
    .access
    .author();
    let dam_domain::Author::Token(author) = author else {
        panic!("a token authors its items")
    };
    for byte in 12..=20 {
        let kept = Item::propose(
            author.clone(),
            Change::Create {
                after: crate::testing::task(byte),
            },
            Timestamp::UNIX_EPOCH,
            &mut |b| b.fill(byte),
        );
        s.store.put_item(&kept).unwrap();
    }
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    dam.edit_each(&query("path:work/"), retitled("x"), as_one())
        .unwrap();
    let items = s.store.items().unwrap();
    let set = items.iter().find_map(|i| i.set.clone()).unwrap();
    assert!(items.iter().all(|item| item.hash != set));
}
