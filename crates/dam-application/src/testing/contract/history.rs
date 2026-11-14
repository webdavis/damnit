use crate::ports::{CommitRepository, Store};
use dam_domain::{Authored, Change, CommitRecord, Timestamp};

use super::{commit_id, oid, remote, task, token_author};

pub fn commit_repository_contract(commits: &dyn CommitRepository) {
    let here = remote("here");
    let there = remote("there");
    assert!(commits.log().unwrap().is_empty());
    assert!(commits.unpushed(&here).unwrap().is_empty());

    let first = record(1, "first", 1);
    let mut second = record(2, "second", 2);
    second.changes[0].author = token_author();
    commits.commit(&first).unwrap();
    commits.commit(&second).unwrap();

    let log = commits.log().unwrap();
    assert_eq!(
        log.iter().map(|r| r.id.clone()).collect::<Vec<_>>(),
        vec![second.id.clone(), first.id.clone()],
        "the log reads newest first"
    );
    assert_eq!(
        log[0].changes, second.changes,
        "a commit keeps its changes and who made each"
    );
    assert_eq!(log[1].changes, first.changes);

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

pub fn commit_clears_the_stage_contract(store: &dyn Store) {
    let committed = record(1, "committed", 1);
    let left = record(2, "left staged", 2).changes.remove(0);
    store.put_staged(committed.changes[0].clone()).unwrap();
    store.put_staged(left.clone()).unwrap();
    store.commit(&committed).unwrap();
    assert_eq!(
        store.staged().unwrap(),
        vec![left],
        "a commit unstages exactly the oids it committed"
    );
}

pub(super) fn record(byte: u8, message: &str, object: u8) -> CommitRecord {
    CommitRecord {
        id: commit_id(byte),
        message: message.to_string(),
        at: Timestamp::UNIX_EPOCH,
        changes: vec![Authored::by_owner(Change::Create {
            after: task(object, message, ""),
        })],
    }
}
