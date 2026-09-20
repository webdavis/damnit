//! What a sync leaves for the operator to read: conflicts and notices.

use crate::ports::{ConflictRepository, Notice, NoticeRepository, ObjectRepository};

use super::{oid, remote, task};

/// A conflict carries both sides, replaces an earlier one for the same oid,
/// and clears on demand. `objects` supplies the local side the store reads.
pub fn conflict_repository_contract(
    objects: &dyn ObjectRepository,
    conflicts: &dyn ConflictRepository,
) {
    let here = remote("here");
    assert!(conflicts.conflicts().unwrap().is_empty());

    let ours = task(1, "ours", "");
    objects.put(&ours).unwrap();
    let theirs = task(1, "theirs", "");
    conflicts.mark_conflict(&here, &oid(1), &theirs).unwrap();

    let held = conflicts.conflicts().unwrap();
    assert_eq!(held.len(), 1);
    assert_eq!(held[0].oid, oid(1));
    assert_eq!(held[0].remote, here);
    assert_eq!(held[0].ours, ours);
    assert_eq!(held[0].theirs, theirs);

    let later = task(1, "theirs again", "");
    conflicts.mark_conflict(&here, &oid(1), &later).unwrap();
    let held = conflicts.conflicts().unwrap();
    assert_eq!(held.len(), 1, "one oid conflicts once");
    assert_eq!(held[0].theirs, later);

    conflicts.clear_conflict(&oid(1)).unwrap();
    assert!(conflicts.conflicts().unwrap().is_empty());
    conflicts.clear_conflict(&oid(1)).unwrap();
}

/// Notices are a log: they keep insertion order, repeat when repeated, and
/// clear all at once.
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
