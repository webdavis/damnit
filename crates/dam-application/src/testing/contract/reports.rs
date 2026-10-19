use crate::ports::{Conflict, Notice, NoticeRepository, Store};

use super::{oid, remote, task};

pub fn conflict_repository_contract(store: &dyn Store) {
    let here = remote("here");
    assert!(store.conflicts().unwrap().is_empty());

    let ours = task(1, "ours", "");
    store.put(&ours).unwrap();
    let theirs = task(1, "theirs", "");
    store
        .mark_conflict(&Conflict {
            remote: here.clone(),
            oid: oid(1),
            ours: ours.clone(),
            theirs: theirs.clone(),
        })
        .unwrap();

    let held = store.conflicts().unwrap();
    assert_eq!(held.len(), 1);
    assert_eq!(held[0].oid, oid(1));
    assert_eq!(held[0].remote, here);
    assert_eq!(held[0].ours, ours);
    assert_eq!(held[0].theirs, theirs);

    let later = task(1, "theirs again", "");
    store
        .mark_conflict(&Conflict {
            remote: here.clone(),
            oid: oid(1),
            ours: ours.clone(),
            theirs: later.clone(),
        })
        .unwrap();
    let held = store.conflicts().unwrap();
    assert_eq!(held.len(), 1, "one oid conflicts once");
    assert_eq!(held[0].theirs, later);

    store.clear_conflict(&oid(1)).unwrap();
    assert!(store.conflicts().unwrap().is_empty());
    store.clear_conflict(&oid(1)).unwrap();
}

pub fn notice_repository_contract(notices: &dyn NoticeRepository) {
    assert!(notices.notices().unwrap().is_empty());

    let first = Notice::PullFailed {
        remote: remote("here"),
        why: "first".into(),
    };
    let second = Notice::EventCancelled {
        oid: oid(2),
        subject: "standup".into(),
        attached: 3,
    };
    notices.add_notice(&first).unwrap();
    notices.add_notice(&second).unwrap();
    notices.add_notice(&first).unwrap();
    assert_eq!(
        notices.notices().unwrap(),
        vec![first.clone(), second, first],
        "notices keep the order they were recorded in, repeats included"
    );

    notices.clear_notices().unwrap();
    assert!(notices.notices().unwrap().is_empty());
    notices.clear_notices().unwrap();
}
