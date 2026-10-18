use crate::errors::UseCaseError;
use crate::ports::Transactional;

pub(crate) fn atomically<T>(
    transaction: &dyn Transactional,
    work: impl FnOnce() -> Result<T, UseCaseError>,
) -> Result<T, UseCaseError> {
    let mut work = Some(work);
    let mut produced = None;
    transaction.in_transaction(&mut || {
        let run = work.take().expect("a unit of work runs once");
        produced = Some(run()?);
        Ok(())
    })?;
    Ok(produced.expect("a committed unit of work produced its value"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::StoreError;
    use crate::testing::prelude::*;
    use crate::testing::{MemoryStore, oid};
    use dam_domain::{Object, Task};

    #[test]
    fn a_unit_that_succeeds_hands_back_its_value() {
        let store = MemoryStore::new();
        let count = atomically(&store, || {
            store.put(&Object::Task(Task::new(oid(1), "kept")))?;
            Ok(store.all()?.len())
        });
        assert_eq!(count, Ok(1));
        assert!(store.get(&oid(1)).unwrap().is_some());
    }

    #[test]
    fn a_unit_that_fails_takes_its_writes_with_it() {
        let store = MemoryStore::new();
        let failed: Result<(), UseCaseError> = atomically(&store, || {
            store.put(&Object::Task(Task::new(oid(1), "discarded")))?;
            Err(UseCaseError::Store(StoreError::Busy("locked".into())))
        });
        assert_eq!(
            failed,
            Err(UseCaseError::Store(StoreError::Busy("locked".into())))
        );
        assert_eq!(store.get(&oid(1)).unwrap(), None);
    }
}
