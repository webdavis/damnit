use crate::SqliteStore;
use dam_application::{ObjectStore, RemoteName};
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
fn put_get_all_delete() {
    let s = SqliteStore::in_memory().unwrap();
    s.put(&task(1, "")).unwrap();
    s.put(&task(2, "a")).unwrap();
    assert_eq!(s.get(&oid(1)).unwrap(), Some(task(1, "")));
    assert_eq!(s.all().unwrap().len(), 2);
    s.delete(&oid(1)).unwrap();
    assert!(s.get(&oid(1)).unwrap().is_none());
}

#[test]
fn children_of_is_direct_children_only() {
    let s = SqliteStore::in_memory().unwrap();
    s.put(&task(1, "p")).unwrap();
    s.put(&task(2, "p/c")).unwrap();
    s.put(&task(3, "p/c/g")).unwrap();
    let kids = s.children_of(&Path::parse("p").unwrap()).unwrap();
    assert_eq!(kids.len(), 1);
    assert_eq!(kids[0].oid(), &oid(2));
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
fn dependents_of_finds_who_lists_the_oid() {
    let s = SqliteStore::in_memory().unwrap();
    let mut d = Task::new(oid(2), "d");
    d.base.depends.push(oid(1));
    s.put(&task(1, "")).unwrap();
    s.put(&Object::Task(d)).unwrap();
    assert_eq!(s.dependents_of(&oid(1)).unwrap(), vec![oid(2)]);
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
fn commit_clears_only_the_oids_it_touched() {
    let s = SqliteStore::in_memory().unwrap();
    let a = task(1, "");
    let b = task(2, "");
    s.put(&a).unwrap();
    s.put(&b).unwrap();
    s.stage(diff(&oid(1), None, Some(&a)).unwrap()).unwrap();
    s.stage(diff(&oid(2), None, Some(&b)).unwrap()).unwrap();
    s.commit(&record(9, vec![diff(&oid(1), None, Some(&a)).unwrap()]))
        .unwrap();
    let left = s.staged().unwrap();
    assert_eq!(left.len(), 1);
    assert_eq!(left[0].oid, oid(2));
}

#[test]
fn log_is_newest_first_and_unpushed_respects_mark_pushed() {
    let s = SqliteStore::in_memory().unwrap();
    let a = task(1, "");
    s.put(&a).unwrap();
    s.commit(&record(1, vec![diff(&oid(1), None, Some(&a)).unwrap()]))
        .unwrap();
    s.commit(&record(2, vec![])).unwrap();
    let log = s.log().unwrap();
    assert_eq!(log[0].id, CommitId::generate(&mut |x| x.fill(2)));
    let remote = RemoteName("t".into());
    assert_eq!(s.unpushed(&remote).unwrap().len(), 2);
    s.mark_pushed(&remote, &log[1].id).unwrap();
    assert_eq!(s.unpushed(&remote).unwrap().len(), 1);
}

#[test]
fn unstage_one_and_all() {
    let s = SqliteStore::in_memory().unwrap();
    for b in [1, 2] {
        let t = task(b, "");
        s.put(&t).unwrap();
        s.stage(diff(&oid(b), None, Some(&t)).unwrap()).unwrap();
    }
    s.unstage(&oid(1)).unwrap();
    assert_eq!(s.staged().unwrap().len(), 1);
    s.unstage_all().unwrap();
    assert!(s.staged().unwrap().is_empty());
}
