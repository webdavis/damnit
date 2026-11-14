use dam_domain::{Date, Day};

use crate::ports::{DayRepository, StoreError};

use super::MemoryStore;

impl DayRepository for MemoryStore {
    fn day(&self, on: Date) -> Result<Option<Day>, StoreError> {
        Ok(self.0.borrow().days.get(&on).cloned())
    }
    fn put_day(&self, day: &Day) -> Result<(), StoreError> {
        self.0.borrow_mut().days.insert(day.day, day.clone());
        Ok(())
    }
    fn days(&self) -> Result<Vec<Day>, StoreError> {
        Ok(self.0.borrow().days.values().cloned().collect())
    }
}

#[cfg(test)]
mod tests;
