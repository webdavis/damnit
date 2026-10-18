use std::cell::Cell;

use dam_domain::{Object, Oid, Path};

use crate::errors::UseCaseError;
use crate::ports::{ObjectRepository, StoreError, Transactional};
use crate::testing::MemoryStore;

mod history;
mod remote_state;

pub struct FaultyStore {
    pub inner: MemoryStore,
    writes_before_failing: Cell<usize>,
    inside_a_unit_of_work: Cell<bool>,
}

impl FaultyStore {
    pub fn new(inner: MemoryStore) -> FaultyStore {
        FaultyStore {
            inner,
            writes_before_failing: Cell::new(usize::MAX),
            inside_a_unit_of_work: Cell::new(false),
        }
    }

    pub fn fail_at_write(&self, index: usize) {
        self.writes_before_failing.set(index);
    }

    pub fn assert_unchanged_when_any_write_fails(
        &self,
        run: impl Fn() -> Result<(), UseCaseError>,
    ) {
        let before = self.inner.clone();
        for index in 0.. {
            self.fail_at_write(index);
            match run() {
                Err(UseCaseError::Store(StoreError::Busy(_))) => assert_eq!(
                    self.inner, before,
                    "the store changed after write {index} failed"
                ),
                Ok(()) => {
                    assert!(index > 0, "the use case wrote nothing");
                    return;
                }
                Err(other) => panic!("write {index}: {other:?}"),
            }
        }
    }

    fn write(&self) -> Result<(), StoreError> {
        if !self.inside_a_unit_of_work.get() {
            return Err(StoreError::Failed("a write outside a unit of work".into()));
        }
        match self.writes_before_failing.get() {
            0 => Err(StoreError::Busy("database is locked".into())),
            left => {
                self.writes_before_failing.set(left - 1);
                Ok(())
            }
        }
    }
}

impl ObjectRepository for FaultyStore {
    fn get(&self, oid: &Oid) -> Result<Option<Object>, StoreError> {
        self.inner.get(oid)
    }
    fn all(&self) -> Result<Vec<Object>, StoreError> {
        self.inner.all()
    }
    fn children_of(&self, path: &Path) -> Result<Vec<Object>, StoreError> {
        self.inner.children_of(path)
    }
    fn dependents_of(&self, oid: &Oid) -> Result<Vec<Oid>, StoreError> {
        self.inner.dependents_of(oid)
    }
    fn put(&self, object: &Object) -> Result<(), StoreError> {
        self.write()?;
        self.inner.put(object)
    }
    fn delete(&self, oid: &Oid) -> Result<(), StoreError> {
        self.write()?;
        self.inner.delete(oid)
    }
    fn committed(&self, oid: &Oid) -> Result<Option<Object>, StoreError> {
        self.inner.committed(oid)
    }
}

impl Transactional for FaultyStore {
    fn in_transaction(
        &self,
        work: &mut dyn FnMut() -> Result<(), UseCaseError>,
    ) -> Result<(), UseCaseError> {
        let outer = self.inside_a_unit_of_work.replace(true);
        let done = self.inner.in_transaction(work);
        self.inside_a_unit_of_work.set(outer);
        done
    }
}
