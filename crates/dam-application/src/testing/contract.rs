//! The behavioral contract every repository implementation owes, so the
//! in-memory double and the durable store are held to one specification
//! rather than to whatever each crate's own tests happen to assert.

use dam_domain::{Change, CommitId, CommitRecord, Object, Oid, Op, Path, Task, Timestamp};

use crate::errors::UseCaseError;
use crate::ports::{
    CommitRepository, ConflictRepository, Notice, NoticeRepository, ObjectRepository, RemoteName,
    RemoteTrackingRepository, StageRepository, StoreError, Transactional,
};

fn oid(byte: u8) -> Oid {
    Oid::generate(&mut |b: &mut [u8]| b.fill(byte))
}

fn commit_id(byte: u8) -> CommitId {
    CommitId::generate(&mut |b: &mut [u8]| b.fill(byte))
}

fn task(byte: u8, subject: &str, path: &str) -> Object {
    let mut t = Task::new(oid(byte), subject);
    t.base.path = Path::parse(path).expect("the contract's own paths parse");
    Object::Task(t)
}

fn remote(name: &str) -> RemoteName {
    RemoteName(name.to_string())
}

/// Reads and writes of the working layer: replacement, deletion, the tree
/// query, the dependency query, and the committed view that `put` never moves.
pub fn object_repository_contract(objects: &dyn ObjectRepository) {
    assert_eq!(objects.get(&oid(1)).unwrap(), None, "absent reads as none");
    assert!(
        objects.all().unwrap().is_empty(),
        "an empty store lists none"
    );

    let parent = task(1, "parent", "work");
    objects.put(&parent).unwrap();
    assert_eq!(objects.get(&oid(1)).unwrap(), Some(parent.clone()));

    let renamed = task(1, "renamed", "work");
    objects.put(&renamed).unwrap();
    assert_eq!(
        objects.get(&oid(1)).unwrap(),
        Some(renamed),
        "a second put replaces rather than duplicates"
    );
    assert_eq!(objects.all().unwrap().len(), 1);

    objects.put(&task(2, "child", "work/parent")).unwrap();
    objects
        .put(&task(4, "grandchild", "work/parent/deep"))
        .unwrap();
    objects.put(&task(5, "elsewhere", "workshop")).unwrap();
    assert_eq!(
        oids_of(objects.children_of(&Path::parse("work").unwrap()).unwrap()),
        vec![oid(2)],
        "children_of answers with the immediate children alone: not the node \
         itself, not a grandchild, and not a path that merely shares a prefix"
    );
    assert_eq!(
        oids_of(
            objects
                .children_of(&Path::parse("work/parent").unwrap())
                .unwrap()
        ),
        vec![oid(4)]
    );

    let mut dependent = task(3, "dependent", "work");
    dependent.base_mut().depends.push(oid(1));
    objects.put(&dependent).unwrap();
    assert_eq!(objects.dependents_of(&oid(1)).unwrap(), vec![oid(3)]);
    assert!(objects.dependents_of(&oid(2)).unwrap().is_empty());

    assert_eq!(
        objects.committed(&oid(1)).unwrap(),
        None,
        "putting into working never touches the committed view"
    );

    let before = objects.all().unwrap().len();
    objects.delete(&oid(3)).unwrap();
    assert_eq!(objects.get(&oid(3)).unwrap(), None);
    objects.delete(&oid(3)).unwrap();
    assert_eq!(
        objects.all().unwrap().len(),
        before - 1,
        "deleting what is already gone is not an error and removes nothing else"
    );
}

fn oids_of(objects: Vec<Object>) -> Vec<Oid> {
    let mut out: Vec<Oid> = objects.iter().map(|o| o.oid().clone()).collect();
    out.sort();
    out
}

/// The stage holds at most one change per oid, coalescing a second one onto
/// the first, and a create followed by a delete leaves nothing staged.
pub fn stage_repository_contract(stage: &dyn StageRepository) {
    assert!(stage.staged().unwrap().is_empty());

    let created = task(1, "created", "");
    stage
        .stage(Change {
            oid: oid(1),
            op: Op::Create,
            before: None,
            after: Some(created.clone()),
        })
        .unwrap();
    assert_eq!(stage.staged().unwrap().len(), 1);

    let edited = task(1, "edited", "");
    stage
        .stage(Change {
            oid: oid(1),
            op: Op::Update,
            before: Some(created.clone()),
            after: Some(edited.clone()),
        })
        .unwrap();
    let staged = stage.staged().unwrap();
    assert_eq!(staged.len(), 1, "one oid stages once");
    assert_eq!(staged[0].op, Op::Create, "a create absorbs a later update");
    assert_eq!(staged[0].after, Some(edited.clone()));

    stage
        .stage(Change {
            oid: oid(1),
            op: Op::Delete,
            before: Some(edited),
            after: None,
        })
        .unwrap();
    assert!(
        stage.staged().unwrap().is_empty(),
        "a create then a delete stages nothing"
    );

    stage
        .stage(Change {
            oid: oid(2),
            op: Op::Create,
            before: None,
            after: Some(task(2, "other", "")),
        })
        .unwrap();
    stage.unstage(&oid(1)).unwrap();
    assert_eq!(
        stage.staged().unwrap().len(),
        1,
        "unstaging an oid that is not staged leaves the others alone"
    );
    stage.unstage_all().unwrap();
    assert!(stage.staged().unwrap().is_empty());
}

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

fn record(byte: u8, message: &str, object: u8) -> CommitRecord {
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
}

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

/// A unit of work lands whole or not at all, across record families, and a
/// failure comes back to the caller unchanged.
pub fn transactional_contract(
    objects: &dyn ObjectRepository,
    stage: &dyn StageRepository,
    transaction: &dyn Transactional,
) {
    transaction
        .in_transaction(&mut || {
            objects.put(&task(1, "kept", ""))?;
            stage.stage(Change {
                oid: oid(1),
                op: Op::Create,
                before: None,
                after: Some(task(1, "kept", "")),
            })?;
            Ok(())
        })
        .expect("work that succeeds commits");
    assert!(objects.get(&oid(1)).unwrap().is_some());
    assert_eq!(stage.staged().unwrap().len(), 1);

    let failed = transaction.in_transaction(&mut || {
        objects.put(&task(2, "discarded", ""))?;
        stage.unstage_all()?;
        Err(UseCaseError::Store(StoreError::Failed("stopped".into())))
    });
    assert_eq!(
        failed,
        Err(UseCaseError::Store(StoreError::Failed("stopped".into()))),
        "the reason the work stopped reaches the caller unchanged"
    );
    assert_eq!(
        objects.get(&oid(2)).unwrap(),
        None,
        "a write made before the failure is rolled back"
    );
    assert_eq!(
        stage.staged().unwrap().len(),
        1,
        "and so is a write to another record family in the same unit"
    );
}
