use std::collections::BTreeSet;

use dam_domain::{Change, Oid};

use crate::errors::{Refusal, UseCaseError};
use crate::ports::{CommitRepository, ObjectRepository, StageRepository};
use crate::use_cases::atomically::atomically;

pub fn add(
    objects: &dyn ObjectRepository,
    stage: &dyn StageRepository,
    oids: &[Oid],
) -> Result<Vec<Change>, UseCaseError> {
    atomically(objects, || stage_each(objects, stage, oids))
}

fn stage_each(
    objects: &dyn ObjectRepository,
    stage: &dyn StageRepository,
    oids: &[Oid],
) -> Result<Vec<Change>, UseCaseError> {
    let mut staged = Vec::new();
    for oid in oids {
        let working = objects.get(oid)?;
        let committed = objects.committed(oid)?;
        if working.is_none() && committed.is_none() {
            return Err(Refusal::NoSuchObject(oid.short().to_string()).into());
        }
        match Change::between(committed, working) {
            Some(change) => {
                stage.put_staged(change.clone())?;
                staged.push(change);
            }
            None => stage.unstage(oid)?,
        }
    }
    Ok(staged)
}

pub(crate) fn working_and_ever_committed_oids(
    objects: &dyn ObjectRepository,
    commits: &dyn CommitRepository,
) -> Result<BTreeSet<Oid>, UseCaseError> {
    let mut oids: BTreeSet<Oid> = objects
        .all()?
        .into_iter()
        .map(|o| o.oid().clone())
        .collect();
    for record in commits.log()? {
        for change in record.changes {
            oids.insert(change.oid().clone());
        }
    }
    Ok(oids)
}

pub fn add_all(
    objects: &dyn ObjectRepository,
    stage: &dyn StageRepository,
    commits: &dyn CommitRepository,
) -> Result<Vec<Change>, UseCaseError> {
    atomically(objects, || {
        let list: Vec<Oid> = working_and_ever_committed_oids(objects, commits)?
            .into_iter()
            .collect();
        stage_each(objects, stage, &list)
    })
}

pub fn reset(stage: &dyn StageRepository, oids: &[Oid]) -> Result<(), UseCaseError> {
    if oids.is_empty() {
        stage.unstage_all()?;
    }
    for oid in oids {
        stage.unstage(oid)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::prelude::*;
    use crate::testing::{FaultyStore, MemoryStore, oid};
    use dam_domain::{Object, Op, Task};

    fn seed(store: &MemoryStore, byte: u8) -> Oid {
        store
            .put(&Object::Task(Task::new(oid(byte), format!("t{byte}"))))
            .unwrap();
        oid(byte)
    }

    #[test]
    fn add_stages_a_create_for_a_new_working_object() {
        let store = MemoryStore::new();
        let id = seed(&store, 1);
        let staged = add(&store, &store, &[id]).unwrap();
        assert_eq!(staged.len(), 1);
        assert_eq!(staged[0].op(), Op::Create);
        assert_eq!(store.staged().unwrap().len(), 1);
    }

    #[test]
    fn add_skips_an_unchanged_object() {
        let store = MemoryStore::new();
        let id = seed(&store, 1);
        let record = dam_domain::CommitRecord {
            id: dam_domain::CommitId::generate(&mut |b| b.fill(9)),
            message: "m".into(),
            at: jiff::Timestamp::UNIX_EPOCH,
            changes: add(&store, &store, std::slice::from_ref(&id)).unwrap(),
        };
        store.commit(&record).unwrap();
        assert!(add(&store, &store, &[id]).unwrap().is_empty());
    }

    #[test]
    fn add_unstages_an_object_edited_back_to_its_committed_state() {
        let store = MemoryStore::new();
        let id = seed(&store, 1);
        let record = dam_domain::CommitRecord {
            id: dam_domain::CommitId::generate(&mut |b| b.fill(9)),
            message: "m".into(),
            at: jiff::Timestamp::UNIX_EPOCH,
            changes: add(&store, &store, std::slice::from_ref(&id)).unwrap(),
        };
        store.commit(&record).unwrap();
        store
            .put(&Object::Task(Task::new(id.clone(), "edited")))
            .unwrap();
        add(&store, &store, std::slice::from_ref(&id)).unwrap();
        store
            .put(&Object::Task(Task::new(id.clone(), "t1")))
            .unwrap();
        assert!(
            add(&store, &store, std::slice::from_ref(&id))
                .unwrap()
                .is_empty()
        );
        assert!(store.staged().unwrap().is_empty());
    }

    #[test]
    fn add_all_includes_deletes_of_committed_objects_gone_from_working() {
        let store = MemoryStore::new();
        let id = seed(&store, 1);
        let record = dam_domain::CommitRecord {
            id: dam_domain::CommitId::generate(&mut |b| b.fill(9)),
            message: "m".into(),
            at: jiff::Timestamp::UNIX_EPOCH,
            changes: add(&store, &store, std::slice::from_ref(&id)).unwrap(),
        };
        store.commit(&record).unwrap();
        store.delete(&id).unwrap();
        let staged = add_all(&store, &store, &store).unwrap();
        assert_eq!(staged[0].op(), Op::Delete);
    }

    #[test]
    fn reset_unstages_one_or_all() {
        let store = MemoryStore::new();
        let a = seed(&store, 1);
        let b = seed(&store, 2);
        add_all(&store, &store, &store).unwrap();
        reset(&store, &[a]).unwrap();
        assert_eq!(store.staged().unwrap().len(), 1);
        reset(&store, &[]).unwrap();
        assert!(store.staged().unwrap().is_empty());
        let _ = b;
    }

    #[test]
    fn add_of_an_unknown_oid_is_refused() {
        let store = MemoryStore::new();
        let err = add(&store, &store, &[oid(7)]).unwrap_err();
        assert!(matches!(
            err,
            UseCaseError::Refused(Refusal::NoSuchObject(_))
        ));
    }

    #[test]
    fn add_that_fails_between_two_objects_stages_neither() {
        let store = FaultyStore::new(MemoryStore::new());
        let a = seed(&store.inner, 1);
        let b = seed(&store.inner, 2);
        store.assert_unchanged_when_any_write_fails(|| {
            add(&store, &store, &[a.clone(), b.clone()]).map(drop)
        });
    }

    #[test]
    fn add_all_that_fails_between_two_objects_stages_neither() {
        let store = FaultyStore::new(MemoryStore::new());
        seed(&store.inner, 1);
        seed(&store.inner, 2);
        store.assert_unchanged_when_any_write_fails(|| add_all(&store, &store, &store).map(drop));
    }
}
