use std::collections::BTreeSet;

use dam_domain::{Change, Oid, diff};

use crate::errors::{Refusal, UseCaseError};
use crate::ports::{CommitRepository, ObjectRepository, StageRepository};

pub fn add(
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
        if let Some(change) = diff(oid, committed.as_ref(), working.as_ref()) {
            stage.stage(change.clone())?;
            staged.push(change);
        }
    }
    Ok(staged)
}

/// Every oid that could still need staging: every working object, plus every
/// oid any commit has ever touched (so a committed object deleted from
/// working, but not yet staged, is not missed).
pub(crate) fn tracked_oids(
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
            oids.insert(change.oid);
        }
    }
    Ok(oids)
}

/// Every working object that differs from committed, plus every committed
/// object missing from working, staged as a delete.
pub fn add_all(
    objects: &dyn ObjectRepository,
    stage: &dyn StageRepository,
    commits: &dyn CommitRepository,
) -> Result<Vec<Change>, UseCaseError> {
    let list: Vec<Oid> = tracked_oids(objects, commits)?.into_iter().collect();
    add(objects, stage, &list)
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
    use crate::testing::{MemoryStore, oid};
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
        assert_eq!(staged[0].op, Op::Create);
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
        assert_eq!(staged[0].op, Op::Delete);
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
}
