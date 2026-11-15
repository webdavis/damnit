use std::collections::BTreeMap;

use dam_domain::{Authored, Change, Channel, Details, ItemHash, Object, Timestamp};

use super::{oid, task, token_author};
use crate::ports::{FollowOn, FollowOnRepository, ObjectState};

fn state(working: Object, owners_staged: Option<Authored>) -> ObjectState {
    ObjectState {
        working,
        owners_staged,
    }
}

fn details() -> Details {
    Details {
        meta: BTreeMap::from([("session".into(), "s1".into())]),
        host: Some("host".into()),
        dir: None,
        repo: None,
        branch: None,
        at: Timestamp::UNIX_EPOCH,
        channel: Channel::Local,
    }
}

fn moved_up() -> FollowOn {
    let before = task(2, "book flights", "home/trip");
    let after = task(2, "book flights", "home");
    let staged = Authored::by_owner(Change::Update {
        before: before.clone(),
        after: after.clone(),
    })
    .with_details(Some(&details()));
    FollowOn {
        before: state(before, None),
        after: state(after, Some(staged)),
    }
}

fn kept_a_token_change() -> FollowOn {
    let staged = Authored {
        change: Change::Create {
            after: task(3, "pack", ""),
        },
        author: token_author(),
        details: None,
    };
    FollowOn {
        before: state(task(3, "pack", ""), Some(staged.clone())),
        after: state(task(3, "pack", "home"), Some(staged)),
    }
}

pub fn follow_on_repository_contract(store: &dyn FollowOnRepository) {
    let item = ItemHash::new(oid(9));
    let other = ItemHash::new(oid(8));
    assert!(store.follow_ons(&item).unwrap().is_empty());

    store
        .keep_follow_ons(&item, &[moved_up(), kept_a_token_change()])
        .unwrap();
    store.keep_follow_ons(&other, &[moved_up()]).unwrap();
    assert_eq!(
        store.follow_ons(&item).unwrap(),
        vec![moved_up(), kept_a_token_change()],
        "follow-ons read back whole, in the order kept"
    );

    store
        .keep_follow_ons(&item, &[kept_a_token_change()])
        .unwrap();
    assert_eq!(
        store.follow_ons(&item).unwrap(),
        vec![kept_a_token_change()],
        "keeping again replaces what the item held"
    );

    store.forget_follow_ons(&item).unwrap();
    assert!(store.follow_ons(&item).unwrap().is_empty());
    assert_eq!(store.follow_ons(&other).unwrap(), vec![moved_up()]);
}
