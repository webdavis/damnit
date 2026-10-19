use dam_domain::{CommitId, CommitRecord, Object, Task, Timestamp};

use super::*;
use crate::testing::prelude::*;
use crate::testing::{MemoryStore, oid};

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
    assert_eq!(resolve_object(&store, oid(0xab).as_str()), Ok(oid(0xab)));
    assert_eq!(resolve_object(&store, "abab"), Ok(oid(0xab)));
}

#[test]
fn a_full_oid_resolves_without_a_scan_so_one_only_history_holds_is_reached() {
    assert_eq!(
        resolve_object(&MemoryStore::new(), oid(0xcd).as_str()),
        Ok(oid(0xcd))
    );
}

#[test]
fn a_short_or_non_hex_prefix_is_not_an_oid() {
    let store = store_holding(&[twin(1)]);
    for text in ["ab", "zzzz", "f5"] {
        assert_eq!(
            resolve_object(&store, text),
            Err(UseCaseError::NotAnOid(text.into()))
        );
    }
    assert_eq!(
        UseCaseError::NotAnOid("f5".into()).to_string(),
        "\"f5\" is not an oid; give at least 4 hex characters"
    );
}

#[test]
fn a_prefix_naming_no_working_object_is_refused_by_the_layer_it_read() {
    let store = store_holding(&[twin(1)]);
    assert_eq!(
        resolve_object(&store, "cdcd"),
        Err(Refusal::NoWorkingObject("cdcd".into()).into())
    );
}

#[test]
fn a_prefix_naming_two_objects_lists_both() {
    let store = store_holding(&[twin(1), twin(2)]);
    assert_eq!(
        resolve_object(&store, "abab"),
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
    assert_eq!(resolve_commit(&store, "abab"), Ok(commit_numbered(1)));
}

#[test]
fn a_commit_prefix_naming_two_commits_lists_both_newest_first() {
    let store = history_of(&[commit_numbered(1), commit_numbered(2)]);
    assert_eq!(
        resolve_commit(&store, "abab"),
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
    assert_eq!(
        resolve_commit(&store, "cdcd"),
        Err(Refusal::NoSuchObject("cdcd".into()).into())
    );
    assert_eq!(
        resolve_commit(&store, "ab"),
        Err(UseCaseError::NotAnOid("ab".into()))
    );
}

#[test]
fn an_ambiguous_prefix_names_each_match_by_its_short_id() {
    let refused = Refusal::AmbiguousPrefix {
        text: "abab".into(),
        matches: vec![
            oid(0xab).to_string(),
            Oid::generate(&mut |x: &mut [u8]| {
                x.fill(0);
                x[..2].fill(0xab);
            })
            .to_string(),
        ],
    };
    assert_eq!(
        refused.to_string(),
        "abab matches more than one id: abababa, abab000; give more characters"
    );
}
