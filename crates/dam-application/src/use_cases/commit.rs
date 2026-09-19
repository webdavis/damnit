use dam_domain::{CommitId, CommitRecord};

use crate::errors::UseCaseError;
use crate::ports::{Clock, ObjectStore, Randomness};

pub fn commit(
    store: &dyn ObjectStore,
    clock: &dyn Clock,
    random: &mut dyn Randomness,
    message: &str,
) -> Result<CommitRecord, UseCaseError> {
    let changes = store.staged()?;
    if changes.is_empty() {
        return Err(UseCaseError::Parse("nothing to commit".into()));
    }
    let record = CommitRecord {
        id: CommitId::generate(&mut |b| random.fill(b)),
        message: message.to_string(),
        at: clock.now(),
        changes,
    };
    store.commit(&record)?;
    Ok(record)
}

pub fn log(store: &dyn ObjectStore) -> Result<Vec<CommitRecord>, UseCaseError> {
    Ok(store.log()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{FixedClock, FixedRandom, MemoryStore, oid};
    use crate::use_cases::stage::add_all;
    use dam_domain::{Object, Task};
    use jiff::civil::date;

    #[test]
    fn commit_records_the_stage_and_empties_it() {
        let store = MemoryStore::new();
        store.put(&Object::Task(Task::new(oid(1), "a"))).unwrap();
        add_all(&store).unwrap();
        let rec = commit(
            &store,
            &FixedClock(date(2026, 9, 18)),
            &mut FixedRandom(5),
            "first",
        )
        .unwrap();
        assert_eq!(rec.message, "first");
        assert_eq!(rec.changes.len(), 1);
        assert!(store.staged().unwrap().is_empty());
        assert_eq!(
            store.committed(&oid(1)).unwrap().unwrap().base().subject,
            "a"
        );
        assert_eq!(log(&store).unwrap()[0].id, rec.id);
    }

    #[test]
    fn an_empty_stage_cannot_be_committed() {
        let store = MemoryStore::new();
        let err = commit(
            &store,
            &FixedClock(date(2026, 9, 18)),
            &mut FixedRandom(5),
            "x",
        )
        .unwrap_err();
        assert_eq!(err, UseCaseError::Parse("nothing to commit".into()));
    }
}
