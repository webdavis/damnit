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
