use dam_domain::{CommitId, CommitRecord, Object, Task, Timestamp};

use super::*;
use crate::testing::prelude::*;
use crate::testing::{Fixture, MemoryStore, oid};

fn twin(last: u8) -> Oid {
    Oid::generate(&mut |x: &mut [u8]| {
        x.fill(0xab);
        x[19] = last;
    })
}

fn store_holding(oids: &[Oid]) -> MemoryStore {
    let store = MemoryStore::new();
    for (i, o) in oids.iter().enumerate() {
        store
            .put(&Object::Task(Task::new(o.clone(), format!("t{i}"))))
            .unwrap();
    }
    store
}

#[test]
fn a_full_oid_and_a_unique_prefix_resolve() {
    let store = store_holding(&[oid(0xab), oid(0xac)]);
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    assert_eq!(dam.resolve_object(oid(0xab).as_str()), Ok(oid(0xab)));
    assert_eq!(dam.resolve_object("abab"), Ok(oid(0xab)));
}

#[test]
fn a_full_oid_resolves_without_a_scan_so_one_only_history_holds_is_reached() {
    let store = MemoryStore::new();
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    assert_eq!(dam.resolve_object(oid(0xcd).as_str()), Ok(oid(0xcd)));
}

#[test]
fn a_short_or_non_hex_prefix_is_not_an_oid() {
    let store = store_holding(&[twin(1)]);
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    for text in ["ab", "zzzz", "f5"] {
        assert_eq!(
            dam.resolve_object(text),
            Err(UseCaseError::NotAnOid(text.into()))
        );
    }
}

#[test]
fn a_prefix_naming_no_working_object_is_refused_by_the_layer_it_read() {
    let store = store_holding(&[twin(1)]);
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    assert_eq!(
        dam.resolve_object("cdcd"),
        Err(Refusal::NoWorkingObject("cdcd".into()).into())
    );
}

#[test]
fn a_prefix_naming_two_objects_lists_both() {
    let store = store_holding(&[twin(1), twin(2)]);
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    assert_eq!(
        dam.resolve_object("abab"),
        Err(Refusal::AmbiguousPrefix {
            text: "abab".into(),
            matches: vec![twin(1).to_string(), twin(2).to_string()],
        }
        .into())
    );
}

fn commit_numbered(last: u8) -> CommitRecord {
    CommitRecord {
        id: CommitId::generate(&mut |x: &mut [u8]| {
            x.fill(0xab);
            x[19] = last;
        }),
        message: format!("c{last}"),
        at: Timestamp::UNIX_EPOCH,
        changes: vec![],
    }
}

fn history_of(commits: &[CommitRecord]) -> MemoryStore {
    let store = MemoryStore::new();
    for c in commits {
        store.commit(c).unwrap();
    }
    store
}

#[test]
fn a_commit_prefix_naming_one_commit_resolves_to_it() {
    let store = history_of(&[commit_numbered(1)]);
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    assert_eq!(dam.resolve_commit("abab"), Ok(commit_numbered(1)));
}

#[test]
fn a_commit_prefix_naming_two_commits_lists_both_newest_first() {
    let store = history_of(&[commit_numbered(1), commit_numbered(2)]);
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    assert_eq!(
        dam.resolve_commit("abab"),
        Err(Refusal::AmbiguousPrefix {
            text: "abab".into(),
            matches: vec![
                commit_numbered(2).id.to_string(),
                commit_numbered(1).id.to_string()
            ],
        }
        .into())
    );
}

#[test]
fn a_commit_prefix_naming_nothing_or_too_short_is_refused() {
    let store = history_of(&[commit_numbered(1)]);
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    assert_eq!(
        dam.resolve_commit("cdcd"),
        Err(Refusal::NoSuchObject("cdcd".into()).into())
    );
    assert_eq!(
        dam.resolve_commit("ab"),
        Err(UseCaseError::NotAnOid("ab".into()))
    );
}
