//! Commits and what each remote has been told about them.

use crate::ports::{CommitRepository, RemoteTrackingRepository};
use dam_domain::{Change, CommitRecord, Op, Timestamp};

use super::{commit_id, oid, remote, task};

/// Commits are ordered newest first, unpushed is per remote, and the retry
/// set is replaced rather than appended to.
pub fn commit_repository_contract(commits: &dyn CommitRepository) {
    let here = remote("here");
    let there = remote("there");
    assert!(commits.log().unwrap().is_empty());
    assert!(commits.unpushed(&here).unwrap().is_empty());

    let first = record(1, "first", 1);
    let second = record(2, "second", 2);
    commits.commit(&first).unwrap();
    commits.commit(&second).unwrap();

    let log = commits.log().unwrap();
    assert_eq!(
        log.iter().map(|r| r.id.clone()).collect::<Vec<_>>(),
        vec![second.id.clone(), first.id.clone()],
        "the log reads newest first"
    );
    assert_eq!(log[0].changes, second.changes, "a commit keeps its changes");

    assert_eq!(commits.unpushed(&here).unwrap().len(), 2);
    commits.mark_pushed(&here, &first.id).unwrap();
    assert_eq!(
        commits
            .unpushed(&here)
            .unwrap()
            .iter()
            .map(|r| r.id.clone())
            .collect::<Vec<_>>(),
        vec![second.id.clone()]
    );
    assert_eq!(
        commits.unpushed(&there).unwrap().len(),
        2,
        "one remote taking a commit says nothing about another"
    );
    commits.mark_pushed(&here, &first.id).unwrap();
    assert_eq!(
        commits.unpushed(&here).unwrap().len(),
        1,
        "marking the same commit pushed twice is idempotent"
    );

    assert!(commits.push_retries(&here).unwrap().is_empty());
    commits.set_push_retries(&here, &[oid(1), oid(2)]).unwrap();
    assert_eq!(commits.push_retries(&here).unwrap(), vec![oid(1), oid(2)]);
    commits.set_push_retries(&here, &[oid(3)]).unwrap();
    assert_eq!(
        commits.push_retries(&here).unwrap(),
        vec![oid(3)],
        "the retry set is replaced, not added to"
    );
    assert!(commits.push_retries(&there).unwrap().is_empty());
    commits.set_push_retries(&here, &[]).unwrap();
    assert!(commits.push_retries(&here).unwrap().is_empty());
}

pub(super) fn record(byte: u8, message: &str, object: u8) -> CommitRecord {
    CommitRecord {
        id: commit_id(byte),
        message: message.to_string(),
        at: Timestamp::UNIX_EPOCH,
        changes: vec![Change {
            oid: oid(object),
            op: Op::Create,
            before: None,
            after: Some(task(object, message, "")),
        }],
    }
}

/// Remote ids map both ways, snapshots and sync state are per remote, and
/// clearing one mapping leaves every other remote untouched.
pub fn remote_tracking_repository_contract(tracking: &dyn RemoteTrackingRepository) {
    let here = remote("here");
    let there = remote("there");
    assert_eq!(tracking.remote_id(&here, &oid(1)).unwrap(), None);
    assert_eq!(tracking.oid_for_remote_id(&here, "r1").unwrap(), None);

    tracking.map_remote_id(&here, &oid(1), "r1").unwrap();
    assert_eq!(
        tracking.remote_id(&here, &oid(1)).unwrap(),
        Some("r1".to_string())
    );
    assert_eq!(
        tracking.oid_for_remote_id(&here, "r1").unwrap(),
        Some(oid(1))
    );
    assert_eq!(
        tracking.remote_id(&there, &oid(1)).unwrap(),
        None,
        "a mapping belongs to one remote"
    );

    tracking.map_remote_id(&here, &oid(1), "r2").unwrap();
    assert_eq!(
        tracking.remote_id(&here, &oid(1)).unwrap(),
        Some("r2".to_string()),
        "remapping an oid replaces its id"
    );

    let snapshot = task(1, "as the remote has it", "");
    tracking.set_remote_snapshot(&here, &snapshot).unwrap();
    assert_eq!(
        tracking.remote_snapshot(&here, &oid(1)).unwrap(),
        Some(snapshot)
    );
    assert_eq!(tracking.remote_snapshot(&there, &oid(1)).unwrap(), None);

    tracking.map_remote_id(&there, &oid(1), "t1").unwrap();
    tracking.clear_remote_mapping(&here, &oid(1)).unwrap();
    assert_eq!(tracking.remote_id(&here, &oid(1)).unwrap(), None);
    assert_eq!(tracking.remote_snapshot(&here, &oid(1)).unwrap(), None);
    assert_eq!(
        tracking.remote_id(&there, &oid(1)).unwrap(),
        Some("t1".to_string()),
        "clearing one remote's mapping leaves the other's standing"
    );

    assert_eq!(tracking.sync_token(&here).unwrap(), None);
    tracking.set_sync_token(&here, Some("cursor")).unwrap();
    assert_eq!(
        tracking.sync_token(&here).unwrap(),
        Some("cursor".to_string())
    );
    assert_eq!(tracking.sync_token(&there).unwrap(), None);
    tracking.set_sync_token(&here, None).unwrap();
    assert_eq!(
        tracking.sync_token(&here).unwrap(),
        None,
        "setting no token clears the one held"
    );

    assert_eq!(tracking.last_pull(&here).unwrap(), None);
    let at = Timestamp::UNIX_EPOCH;
    tracking.set_last_pull(&here, at).unwrap();
    assert_eq!(tracking.last_pull(&here).unwrap(), Some(at));
    assert_eq!(tracking.last_pull(&there).unwrap(), None);

    assert_eq!(tracking.last_push(&here).unwrap(), None);
    let pushed_at = at + std::time::Duration::from_secs(60);
    tracking.set_last_push(&here, pushed_at).unwrap();
    assert_eq!(tracking.last_push(&here).unwrap(), Some(pushed_at));
    assert_eq!(tracking.last_push(&there).unwrap(), None);
    assert_eq!(
        tracking.last_pull(&here).unwrap(),
        Some(at),
        "the two times are kept apart"
    );
}
