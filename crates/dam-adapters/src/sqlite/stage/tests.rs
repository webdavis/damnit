use crate::SqliteStore;
use dam_application::{CommitRepository, ObjectRepository, RemoteName, StageRepository};
use dam_domain::{CommitId, CommitRecord, Object, Oid, Op, Path, Task, diff};

fn oid(b: u8) -> Oid {
    Oid::generate(&mut |x: &mut [u8]| x.fill(b))
}

fn task(b: u8, path: &str) -> Object {
    let mut t = Task::new(oid(b), format!("t{b}"));
    t.base.path = Path::parse(path).unwrap();
    Object::Task(t)
}

fn record(b: u8, changes: Vec<dam_domain::Change>) -> CommitRecord {
    CommitRecord {
        id: CommitId::generate(&mut |x| x.fill(b)),
        message: "m".into(),
        at: jiff::Timestamp::UNIX_EPOCH,
        changes,
    }
}

#[test]
fn children_of_does_not_treat_underscore_and_percent_as_wildcards() {
    let s = SqliteStore::in_memory().unwrap();
    s.put(&task(1, "a_b/c")).unwrap();
    s.put(&task(2, "axb/c")).unwrap();
    let kids = s.children_of(&Path::parse("a_b").unwrap()).unwrap();
    assert_eq!(kids.len(), 1);
    assert_eq!(kids[0].oid(), &oid(1));
}

#[test]
fn children_of_does_not_panic_on_a_non_matching_multibyte_path() {
    let s = SqliteStore::in_memory().unwrap();
    s.put(&task(1, "éé/c")).unwrap();
    let kids = s.children_of(&Path::parse("__").unwrap()).unwrap();
    assert!(kids.is_empty());
}

#[test]
fn stage_coalesces_and_commit_moves_the_stage_into_history() {
    let s = SqliteStore::in_memory().unwrap();
    let a = task(1, "");
    s.put(&a).unwrap();
    s.stage(diff(&oid(1), None, Some(&a)).unwrap()).unwrap();
    let mut b = a.clone();
    b.base_mut().subject = "b".into();
    s.stage(diff(&oid(1), Some(&a), Some(&b)).unwrap()).unwrap();
    let staged = s.staged().unwrap();
    assert_eq!(staged.len(), 1);
    assert_eq!(staged[0].op, Op::Create);
    assert_eq!(staged[0].after, Some(b.clone()));
    s.commit(&record(9, staged)).unwrap();
    assert!(s.staged().unwrap().is_empty());
    assert_eq!(s.committed(&oid(1)).unwrap(), Some(b));
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
