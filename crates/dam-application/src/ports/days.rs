use dam_domain::{Date, Day};

use super::store::StoreError;

pub trait DayRepository {
    fn day(&self, on: Date) -> Result<Option<Day>, StoreError>;
    fn put_day(&self, day: &Day) -> Result<(), StoreError>;
    fn days(&self) -> Result<Vec<Day>, StoreError>;
}
