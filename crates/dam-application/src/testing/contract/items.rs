use dam_domain::{Change, Channel, Item, ItemHash, Outcome, Timestamp, Token, TokenAuthor};

use super::{oid, task};
use crate::ports::ItemRepository;

fn vpt() -> TokenAuthor {
    TokenAuthor {
        name: "vpt".into(),
        key_id: Token::generate(&mut |b: &mut [u8]| b.fill(3))
            .key_id()
            .clone(),
        channel: Channel::Local,
    }
}

pub fn item_repository_contract(items: &dyn ItemRepository) {
    assert!(items.items().unwrap().is_empty());
    assert_eq!(items.item(&ItemHash::new(oid(1))).unwrap(), None);

    let created = Item::propose(
        vpt(),
        Change::Create {
            after: task(1, "call the dentist", "inbox"),
        },
        Timestamp::UNIX_EPOCH,
        &mut |b| b.fill(1),
    );
    let changed = Item::propose(
        vpt(),
        Change::Update {
            before: task(5, "pay rent", ""),
            after: task(5, "pay rent today", ""),
        },
        Timestamp::UNIX_EPOCH,
        &mut |b| b.fill(2),
    );
    items.put_item(&created).unwrap();
    items.put_item(&changed).unwrap();
    assert_eq!(
        items.items().unwrap(),
        vec![created.clone(), changed.clone()],
        "items read back whole, oldest first"
    );
    assert_eq!(items.item(&changed.hash).unwrap(), Some(changed.clone()));

    let at: Timestamp = "2026-11-11T09:00:00Z".parse().unwrap();
    items.settle(&created.hash, Outcome::Declined(at)).unwrap();
    let settled = items.item(&created.hash).unwrap().unwrap();
    assert_eq!(settled.outcome, Outcome::Declined(at));
    assert_eq!(
        items.items().unwrap().len(),
        2,
        "a settled item is kept, never forgotten"
    );
}
