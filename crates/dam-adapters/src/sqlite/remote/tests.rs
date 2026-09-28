use crate::SqliteStore;
use dam_application::{RemoteName, RemoteTrackingRepository};
use dam_domain::{Object, Oid, Task};

fn oid(b: u8) -> Oid {
    Oid::generate(&mut |x: &mut [u8]| x.fill(b))
}

fn remote() -> RemoteName {
    RemoteName("todoist".into())
}

#[test]
fn remote_ids_map_both_ways_and_overwrite() {
    let s = SqliteStore::in_memory().unwrap();
    s.map_remote_id(&remote(), &oid(1), "r1").unwrap();
    assert_eq!(
        s.remote_id(&remote(), &oid(1)).unwrap().as_deref(),
        Some("r1")
    );
    assert_eq!(s.oid_for_remote_id(&remote(), "r1").unwrap(), Some(oid(1)));
    s.map_remote_id(&remote(), &oid(1), "r2").unwrap();
    assert_eq!(
        s.remote_id(&remote(), &oid(1)).unwrap().as_deref(),
        Some("r2")
    );
    assert!(s.oid_for_remote_id(&remote(), "r1").unwrap().is_none());
}

#[test]
fn snapshots_and_sync_tokens_are_per_remote() {
    let s = SqliteStore::in_memory().unwrap();
    let o = Object::Task(Task::new(oid(1), "x"));
    s.set_remote_snapshot(&remote(), &o).unwrap();
    assert_eq!(s.remote_snapshot(&remote(), &oid(1)).unwrap(), Some(o));
    assert!(
        s.remote_snapshot(&RemoteName("other".into()), &oid(1))
            .unwrap()
            .is_none()
    );
    s.set_sync_token(&remote(), Some("s1")).unwrap();
    assert_eq!(s.sync_token(&remote()).unwrap().as_deref(), Some("s1"));
    s.set_sync_token(&remote(), None).unwrap();
    assert!(s.sync_token(&remote()).unwrap().is_none());
}

#[test]
fn clear_remote_mapping_drops_id_and_snapshot_for_that_remote_only() {
    let s = SqliteStore::in_memory().unwrap();
    let o = Object::Task(Task::new(oid(1), "x"));
    s.map_remote_id(&remote(), &oid(1), "r1").unwrap();
    s.set_remote_snapshot(&remote(), &o).unwrap();
    let other = RemoteName("other".into());
    s.map_remote_id(&other, &oid(1), "r1").unwrap();
    s.set_remote_snapshot(&other, &o).unwrap();
    s.clear_remote_mapping(&remote(), &oid(1)).unwrap();
    assert!(s.remote_id(&remote(), &oid(1)).unwrap().is_none());
    assert!(s.remote_snapshot(&remote(), &oid(1)).unwrap().is_none());
    assert_eq!(s.remote_id(&other, &oid(1)).unwrap().as_deref(), Some("r1"));
    assert_eq!(s.remote_snapshot(&other, &oid(1)).unwrap(), Some(o));
}

#[test]
fn last_push_is_absent_then_set_and_does_not_disturb_last_pull() {
    let s = SqliteStore::in_memory().unwrap();
    assert!(s.last_push(&remote()).unwrap().is_none());
    let pulled = jiff::Timestamp::UNIX_EPOCH;
    let pushed = pulled + std::time::Duration::from_secs(60);
    s.set_last_pull(&remote(), pulled).unwrap();
    s.set_last_push(&remote(), pushed).unwrap();
    assert_eq!(s.last_push(&remote()).unwrap(), Some(pushed));
    assert_eq!(s.last_pull(&remote()).unwrap(), Some(pulled));
    assert!(
        s.last_push(&RemoteName("other".into())).unwrap().is_none(),
        "the record is per remote"
    );
}

#[test]
fn last_pull_is_absent_then_set() {
    let s = SqliteStore::in_memory().unwrap();
    assert!(s.last_pull(&remote()).unwrap().is_none());
    s.set_last_pull(&remote(), jiff::Timestamp::UNIX_EPOCH)
        .unwrap();
    assert_eq!(
        s.last_pull(&remote()).unwrap(),
        Some(jiff::Timestamp::UNIX_EPOCH)
    );
}
