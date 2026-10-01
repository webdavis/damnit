use super::*;
use crate::testing::prelude::*;
use crate::testing::{FixedClock, FixedRandom, MemoryStore, oid};
use crate::use_cases::commit::commit;
use crate::use_cases::stage::add;
use dam_domain::{Event, Kind, Object, Op, Task, When};
use jiff::civil::date;

#[test]
fn a_fully_staged_change_is_not_also_unstaged() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    store.put(&Object::Task(Task::new(oid(1), "a"))).unwrap();
    add(&store, &store, &[oid(1)]).unwrap();
    let s = status(repos, &[]).unwrap();
    assert_eq!(s.staged.len(), 1);
    assert!(s.unstaged.is_empty());
}

#[test]
fn an_edit_after_staging_shows_as_unstaged_on_top_of_the_stage() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    store.put(&Object::Task(Task::new(oid(1), "a"))).unwrap();
    add(&store, &store, &[oid(1)]).unwrap();
    let mut t = store.get(&oid(1)).unwrap().unwrap();
    t.base_mut().subject = "b".into();
    store.put(&t).unwrap();
    let s = status(repos, &[]).unwrap();
    assert_eq!(s.staged[0].after.as_ref().unwrap().base().subject, "a");
    assert_eq!(s.unstaged[0].op, Op::Update);
    assert_eq!(s.unstaged[0].after.as_ref().unwrap().base().subject, "b");
}

#[test]
fn unpushed_names_each_commit_newest_first() {
    let store = MemoryStore::new();
    let clock = FixedClock(date(2026, 9, 18));
    let random = FixedRandom::new(5);
    store.put(&Object::Task(Task::new(oid(1), "a"))).unwrap();
    add(&store, &store, &[oid(1)]).unwrap();
    let first = commit(&store, &store, &clock, &random, "first").unwrap();
    store.put(&Object::Task(Task::new(oid(2), "b"))).unwrap();
    add(&store, &store, &[oid(2)]).unwrap();
    let second = commit(&store, &store, &clock, &random, "second").unwrap();
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
    for byte in [1u8, 2] {
        store.put(&Object::Task(Task::new(oid(byte), "a"))).unwrap();
        add(&store, &store, &[oid(byte)]).unwrap();
        commit(&store, &store, &clock, &random, "create").unwrap();
    }
    let mut second = store.get(&oid(2)).unwrap().unwrap();
    second.base_mut().subject = "changed again".into();
    store.put(&second).unwrap();
    add(&store, &store, &[oid(2)]).unwrap();
    commit(&store, &store, &clock, &random, "touch it again").unwrap();

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
    add(&store, &store, &[oid(1)]).unwrap();
    commit(
        &store,
        &store,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(5),
        "m",
    )
    .unwrap();
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
    add(&store, &store, &[oid(1)]).unwrap();
    commit(
        &store,
        &store,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(5),
        "m",
    )
    .unwrap();
    store.delete(&oid(1)).unwrap();
    let s = diff_working(&store, &store, &store).unwrap();
    assert_eq!(s.len(), 1);
    assert_eq!(s[0].op, Op::Delete);
    let staged = crate::use_cases::stage::add_all(&store, &store, &store).unwrap();
    assert_eq!(staged.len(), 1);
    assert_eq!(staged[0].op, Op::Delete);
}

fn commit_one(store: &MemoryStore, object: Object, seed: u8) -> CommitId {
    store.put(&object).unwrap();
    add(store, store, &[object.oid().clone()]).unwrap();
    commit(
        store,
        store,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(seed),
        "m",
    )
    .unwrap()
    .id
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
    add(&store, &store, &[oid(1), oid(2)]).unwrap();
    let both = commit(
        &store,
        &store,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(3),
        "m",
    )
    .unwrap();
    let todoist = RemoteName("todoist".into());
    store.set_accepted_kinds(&todoist, &[Kind::Task]).unwrap();
    let s = status(Repositories::of(&store), std::slice::from_ref(&todoist)).unwrap();
    assert_eq!(s.unpushed[0].commit_ids, vec![both.id]);
    assert_eq!(s.unpushed[0].oids, vec![oid(1)]);
}
