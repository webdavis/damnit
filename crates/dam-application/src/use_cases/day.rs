use dam_domain::{Date, Day};

use crate::errors::UseCaseError;
use crate::service::Dam;

impl Dam<'_> {
    pub fn day(&self, on: Date) -> Result<Day, UseCaseError> {
        Ok(self.store.day(on)?.unwrap_or_else(|| Day::open(on)))
    }

    pub fn record_day(&self, day: Day) -> Result<Day, UseCaseError> {
        self.store.put_day(&day)?;
        Ok(day)
    }

    pub fn days_newest_first(&self) -> Result<Vec<Day>, UseCaseError> {
        let mut days = self.store.days()?;
        days.sort_by_key(|d| std::cmp::Reverse(d.day));
        Ok(days)
    }
}
