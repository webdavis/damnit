use super::*;
use crate::service::Dam;
use crate::testing::prelude::*;
use crate::testing::{FixedClock, FixedRandom, Fixture, MemoryStore, oid};
use dam_domain::{Event, Kind, Object, Op, Task, When};
use jiff::civil::date;

#[test]
fn a_fully_staged_change_is_not_also_unstaged() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    store.put(&Object::Task(Task::new(oid(1), "a"))).unwrap();
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    dam.add(&[oid(1)]).unwrap();
    let s = status(repos, &[]).unwrap();
    assert_eq!(s.staged.len(), 1);
    assert!(s.unstaged.is_empty());
}

#[test]
fn an_edit_after_staging_shows_as_unstaged_on_top_of_the_stage() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    store.put(&Object::Task(Task::new(oid(1), "a"))).unwrap();
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    dam.add(&[oid(1)]).unwrap();
    let mut t = store.get(&oid(1)).unwrap().unwrap();
    t.base_mut().subject = "b".into();
    store.put(&t).unwrap();
    let s = status(repos, &[]).unwrap();
    assert_eq!(s.staged[0].after().unwrap().base().subject, "a");
    assert_eq!(s.unstaged[0].op(), Op::Update);
    assert_eq!(s.unstaged[0].after().unwrap().base().subject, "b");
}

#[test]
fn unpushed_names_each_commit_newest_first() {
    let store = MemoryStore::new();
    let clock = FixedClock(date(2026, 9, 18));
    let random = FixedRandom::new(5);
    store.put(&Object::Task(Task::new(oid(1), "a"))).unwrap();
    let fixture = Fixture::default();
    let dam = Dam {
        clock: &clock,
        random: &random,
        ..fixture.dam(&store)
    };
    dam.add(&[oid(1)]).unwrap();
    let first = dam.commit("first").unwrap();
    store.put(&Object::Task(Task::new(oid(2), "b"))).unwrap();
    dam.add(&[oid(2)]).unwrap();
    let second = dam.commit("second").unwrap();
    let remote = RemoteName("todoist".into());
    let s = status(Repositories::of(&store), std::slice::from_ref(&remote)).unwrap();
    assert_eq!(s.unpushed[0].remote, remote);
    assert_eq!(s.unpushed[0].commit_ids, vec![second.id, first.id]);
}

#[test]
fn unpushed_names_each_object_once_in_first_appearance_order() {
    let store = MemoryStore::new();
    let clock = FixedClock(date(2026, 9, 18));
    let random = FixedRandom::new(5);
    let fixture = Fixture::default();
    let dam = Dam {
        clock: &clock,
        random: &random,
        ..fixture.dam(&store)
    };
    for byte in [1u8, 2] {
        store.put(&Object::Task(Task::new(oid(byte), "a"))).unwrap();
        dam.add(&[oid(byte)]).unwrap();
        dam.commit("create").unwrap();
    }
    let mut second = store.get(&oid(2)).unwrap().unwrap();
    second.base_mut().subject = "changed again".into();
    store.put(&second).unwrap();
    dam.add(&[oid(2)]).unwrap();
    dam.commit("touch it again").unwrap();

    let remote = RemoteName("todoist".into());
    let s = status(Repositories::of(&store), std::slice::from_ref(&remote)).unwrap();
    assert_eq!(s.unpushed[0].commit_ids.len(), 3);
    assert_eq!(s.unpushed[0].oids, vec![oid(2), oid(1)]);
}

#[test]
fn a_remote_owed_nothing_gets_a_row_with_empty_lists() {
    let store = MemoryStore::new();
    let remote = RemoteName("todoist".into());
    let s = status(Repositories::of(&store), std::slice::from_ref(&remote)).unwrap();
    assert_eq!(s.unpushed.len(), 1);
    assert!(s.unpushed[0].commit_ids.is_empty());
    assert!(s.unpushed[0].oids.is_empty());
}

#[test]
fn unpushed_counts_per_remote() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    store.put(&Object::Task(Task::new(oid(1), "a"))).unwrap();
    let fixture = Fixture {
        random: FixedRandom::new(5),
        ..Fixture::default()
    };
    let dam = fixture.dam(&store);
    dam.add(&[oid(1)]).unwrap();
    dam.commit("m").unwrap();
    let remote = RemoteName("todoist".into());
    let s = status(repos, std::slice::from_ref(&remote)).unwrap();
    assert_eq!(s.unpushed.len(), 1);
    assert_eq!(s.unpushed[0].remote, remote);
    assert_eq!(s.unpushed[0].commit_ids.len(), 1);
}

#[test]
fn a_working_delete_not_yet_staged_shows_as_an_unstaged_delete() {
    let store = MemoryStore::new();
    store.put(&Object::Task(Task::new(oid(1), "a"))).unwrap();
    let fixture = Fixture {
        random: FixedRandom::new(5),
        ..Fixture::default()
    };
    let dam = fixture.dam(&store);
    dam.add(&[oid(1)]).unwrap();
    dam.commit("m").unwrap();
    store.delete(&oid(1)).unwrap();
    let s = dam.diff_working().unwrap();
    assert_eq!(s.len(), 1);
    assert_eq!(s[0].op(), Op::Delete);
    let staged = dam.add_all().unwrap();
    assert_eq!(staged.len(), 1);
    assert_eq!(staged[0].op(), Op::Delete);
}

fn commit_one(store: &MemoryStore, object: Object, seed: u8) -> CommitId {
    store.put(&object).unwrap();
    let fixture = Fixture {
        random: FixedRandom::new(seed),
        ..Fixture::default()
    };
    let dam = fixture.dam(store);
    dam.add(&[object.oid().clone()]).unwrap();
    dam.commit("m").unwrap().id
}

fn event(byte: u8) -> Object {
    Object::Event(Event::new(
        oid(byte),
        "e",
        When::Day(date(2026, 9, 18)),
        When::Day(date(2026, 9, 19)),
    ))
}

#[test]
fn a_commit_is_owed_only_to_the_remotes_that_accept_one_of_its_kinds() {
    let store = MemoryStore::new();
    let task_commit = commit_one(&store, Object::Task(Task::new(oid(1), "a")), 1);
    let event_commit = commit_one(&store, event(2), 2);
    let todoist = RemoteName("todoist".into());
    let gcal = RemoteName("gcal".into());
    let never_connected = RemoteName("new".into());
    store.set_accepted_kinds(&todoist, &[Kind::Task]).unwrap();
    store.set_accepted_kinds(&gcal, &[]).unwrap();
    let s = status(Repositories::of(&store), &[todoist, gcal, never_connected]).unwrap();
    assert_eq!(s.unpushed[0].commit_ids, vec![task_commit.clone()]);
    assert_eq!(s.unpushed[0].oids, vec![oid(1)]);
    assert!(s.unpushed[1].commit_ids.is_empty());
    assert!(s.unpushed[1].oids.is_empty());
    assert_eq!(s.unpushed[2].commit_ids, vec![event_commit, task_commit]);
}

#[test]
fn a_mixed_commit_owes_a_remote_only_the_objects_of_kinds_it_accepts() {
    let store = MemoryStore::new();
    store.put(&Object::Task(Task::new(oid(1), "a"))).unwrap();
    store.put(&event(2)).unwrap();
    let fixture = Fixture {
        random: FixedRandom::new(3),
        ..Fixture::default()
    };
    let dam = fixture.dam(&store);
    dam.add(&[oid(1), oid(2)]).unwrap();
    let both = dam.commit("m").unwrap();
    let todoist = RemoteName("todoist".into());
    store.set_accepted_kinds(&todoist, &[Kind::Task]).unwrap();
    let s = status(Repositories::of(&store), std::slice::from_ref(&todoist)).unwrap();
    assert_eq!(s.unpushed[0].commit_ids, vec![both.id]);
    assert_eq!(s.unpushed[0].oids, vec![oid(1)]);
}
