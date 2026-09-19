use std::collections::BTreeSet;

use dam_domain::{Change, Oid, diff};

use crate::errors::{Refusal, UseCaseError};
use crate::ports::ObjectStore;

pub fn add(store: &dyn ObjectStore, oids: &[Oid]) -> Result<Vec<Change>, UseCaseError> {
    let mut staged = Vec::new();
    for oid in oids {
        let working = store.get(oid)?;
        let committed = store.committed(oid)?;
        if working.is_none() && committed.is_none() {
            return Err(Refusal::NoSuchObject(oid.short().to_string()).into());
        }
        if let Some(change) = diff(oid, committed.as_ref(), working.as_ref()) {
            store.stage(change.clone())?;
            staged.push(change);
        }
    }
    Ok(staged)
}

/// Every working object that differs from committed, plus every committed
/// object missing from working, staged as a delete.
pub fn add_all(store: &dyn ObjectStore) -> Result<Vec<Change>, UseCaseError> {
    let mut oids: BTreeSet<Oid> = store.all()?.into_iter().map(|o| o.oid().clone()).collect();
    for record in store.log()? {
        for change in record.changes {
            oids.insert(change.oid);
        }
    }
    let list: Vec<Oid> = oids.into_iter().collect();
    add(store, &list)
}

pub fn reset(store: &dyn ObjectStore, oids: &[Oid]) -> Result<(), UseCaseError> {
    if oids.is_empty() {
        store.unstage_all()?;
    }
    for oid in oids {
        store.unstage(oid)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let staged = add(&store, &[id]).unwrap();
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
            changes: add(&store, std::slice::from_ref(&id)).unwrap(),
        };
        store.commit(&record).unwrap();
        assert!(add(&store, &[id]).unwrap().is_empty());
    }

    #[test]
    fn add_all_includes_deletes_of_committed_objects_gone_from_working() {
        let store = MemoryStore::new();
        let id = seed(&store, 1);
        let record = dam_domain::CommitRecord {
            id: dam_domain::CommitId::generate(&mut |b| b.fill(9)),
            message: "m".into(),
            at: jiff::Timestamp::UNIX_EPOCH,
            changes: add(&store, std::slice::from_ref(&id)).unwrap(),
        };
        store.commit(&record).unwrap();
        store.delete(&id).unwrap();
        let staged = add_all(&store).unwrap();
        assert_eq!(staged[0].op, Op::Delete);
    }

    #[test]
    fn reset_unstages_one_or_all() {
        let store = MemoryStore::new();
        let a = seed(&store, 1);
        let b = seed(&store, 2);
        add_all(&store).unwrap();
        reset(&store, &[a]).unwrap();
        assert_eq!(store.staged().unwrap().len(), 1);
        reset(&store, &[]).unwrap();
        assert!(store.staged().unwrap().is_empty());
        let _ = b;
    }

    #[test]
    fn add_of_an_unknown_oid_is_refused() {
        let store = MemoryStore::new();
        let err = add(&store, &[oid(7)]).unwrap_err();
        assert!(matches!(
            err,
            UseCaseError::Refused(Refusal::NoSuchObject(_))
        ));
    }
}
