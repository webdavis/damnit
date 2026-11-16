use crate::ports::{Conflict, Store};
use dam_domain::{Author, Authored, Change, CommitRecord, Item, Object, Timestamp};

use super::{commit_id, oid, remote, task, token_author};

fn held_by_errands(subject: &str) -> Object {
    let mut object = task(2, subject, "Home/errands");
    object.base_mut().parent = Some(oid(1));
    object
}

pub fn parent_repository_contract(store: &dyn Store) {
    let here = remote("here");
    let held = held_by_errands("milk");

    store.put(&held).unwrap();
    assert_eq!(store.get(&oid(2)).unwrap(), Some(held.clone()), "put");

    let created = Authored::by_owner(Change::Create {
        after: held.clone(),
    });
    store.put_staged(created.clone()).unwrap();
    assert_eq!(store.staged().unwrap(), vec![created.clone()], "stage");

    store
        .commit(&CommitRecord {
            id: commit_id(1),
            message: "milk".into(),
            pulled: false,
            at: Timestamp::UNIX_EPOCH,
            changes: vec![created.clone()],
        })
        .unwrap();
    assert_eq!(store.log().unwrap()[0].changes, vec![created], "commit");
    assert_eq!(store.committed(&oid(2)).unwrap(), Some(held.clone()));

    let mut unsettled = held.clone();
    unsettled.base_mut().parent = None;
    store
        .commit(&CommitRecord {
            id: commit_id(2),
            message: "unsettle".into(),
            pulled: false,
            at: Timestamp::UNIX_EPOCH,
            changes: vec![Authored::by_owner(Change::Update {
                before: held.clone(),
                after: unsettled,
            })],
        })
        .unwrap();
    store.settle_committed_parent(&oid(2), &oid(1)).unwrap();
    assert_eq!(
        store.committed(&oid(2)).unwrap(),
        Some(held.clone()),
        "settled in the committed layer"
    );
    assert_eq!(store.log().unwrap().len(), 2, "settling makes no commit");

    store.set_remote_snapshot(&here, &held).unwrap();
    assert_eq!(
        store.remote_snapshot(&here, &oid(2)).unwrap(),
        Some(held.clone()),
        "a remote's last-seen copy"
    );

    let theirs = held_by_errands("oat milk");
    store
        .mark_conflict(&Conflict {
            remote: here,
            oid: oid(2),
            ours: Some(held.clone()),
            theirs: theirs.clone(),
        })
        .unwrap();
    let conflict = store.conflicts().unwrap().remove(0);
    assert_eq!(
        (conflict.ours, conflict.theirs),
        (Some(held.clone()), theirs)
    );

    let Author::Token(tool) = token_author() else {
        unreachable!("the contract's token author is a token")
    };
    let item = Item::propose(
        tool,
        Change::Create { after: held },
        Timestamp::UNIX_EPOCH,
        &mut |b| b.fill(4),
    );
    store.put_item(&item).unwrap();
    assert_eq!(store.item(&item.hash).unwrap(), Some(item), "a token item");
}
