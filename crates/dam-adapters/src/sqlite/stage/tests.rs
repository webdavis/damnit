use crate::SqliteStore;
use dam_application::{CommitRepository, ObjectRepository, RemoteName, StageRepository};
use dam_domain::{Change, CommitId, CommitRecord, Object, Oid, Path, Task};

fn oid(b: u8) -> Oid {
    Oid::generate(&mut |x: &mut [u8]| x.fill(b))
}

fn task(b: u8, path: &str) -> Object {
    let mut t = Task::new(oid(b), format!("t{b}"));
    t.base.path = Path::parse(path).unwrap();
    Object::Task(t)
}

fn record(b: u8, changes: Vec<Change>) -> CommitRecord {
    CommitRecord {
        id: CommitId::generate(&mut |x| x.fill(b)),
        message: "m".into(),
        at: jiff::Timestamp::UNIX_EPOCH,
        changes,
    }
}

#[test]
fn commit_moves_the_stage_into_history() {
    let s = SqliteStore::in_memory().unwrap();
    let a = task(1, "");
    s.put(&a).unwrap();
    s.put_staged(Change::Create { after: a.clone() }).unwrap();
    s.commit(&record(9, s.staged().unwrap())).unwrap();
    assert!(s.staged().unwrap().is_empty());
    assert_eq!(s.committed(&oid(1)).unwrap(), Some(a));
    assert_eq!(s.log().unwrap().len(), 1);
}

#[test]
fn unpushed_commits_come_oldest_first() {
    let s = SqliteStore::in_memory().unwrap();
    s.commit(&record(2, vec![])).unwrap();
    s.commit(&record(1, vec![])).unwrap();
    let unpushed: Vec<CommitId> = s
        .unpushed(&RemoteName("t".into()))
        .unwrap()
        .into_iter()
        .map(|r| r.id)
        .collect();
    assert_eq!(
        unpushed,
        vec![
            CommitId::generate(&mut |x| x.fill(2)),
            CommitId::generate(&mut |x| x.fill(1)),
        ]
    );
}
