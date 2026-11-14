use dam_domain::{
    Blockers, Change, Channel, Completion, Item, Object, Timestamp, Token, TokenAuthor,
};

use crate::testing::prelude::*;
use crate::testing::{Scenario, oid, task_in};
use crate::use_cases::items::refusal::ItemRefusal;
use crate::use_cases::prefix::Named;

fn tool() -> TokenAuthor {
    TokenAuthor {
        name: "tool".into(),
        key_id: Token::generate(&mut |b: &mut [u8]| b.fill(3))
            .key_id()
            .clone(),
        channel: Channel::Local,
    }
}

#[test]
fn an_item_that_makes_a_task_wait_on_an_open_one_and_completes_it_is_refused() {
    let s = Scenario::default();
    let rent = Object::Task(task_in(1, "pay rent", "home"));
    s.committed(rent.clone())
        .committed(Object::Task(task_in(2, "find the keys", "home")));
    let mut done = task_in(1, "pay rent", "home");
    done.base.depends.push(oid(2));
    done.completion = Completion::Done { at: None };
    let change = Change::Update {
        before: rent,
        after: Object::Task(done),
    };
    let item = Item::propose(tool(), change, Timestamp::UNIX_EPOCH, &mut |b| b.fill(9));
    s.store.put_item(&item).unwrap();

    assert_eq!(
        s.dam().add(&[Named::Item(item.hash.clone())]),
        Err(ItemRefusal::Blocked {
            hash: item.hash.clone(),
            target: oid(1),
            blockers: Blockers {
                dependencies: vec![oid(2)],
                children: Vec::new(),
            },
        }
        .into())
    );
}
