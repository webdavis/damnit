use super::nth_weekday::NthWeekday;
use super::{Rule, RuleError};
use crate::weekday::parse_weekday;

impl Rule {
    pub(super) fn take_on_list(&mut self, list: &str) -> Result<(), RuleError> {
        for item in list.split(',') {
            if let Ok(day) = item.parse::<i8>() {
                if !(1..=31).contains(&day) {
                    return Err(RuleError::BadDay(item.to_string()));
                }
                self.by_month_day.push(day);
            } else if let Some(nth) = NthWeekday::parse(item) {
                self.by_nth_weekday.push(nth?);
            } else {
                self.by_day
                    .push(parse_weekday(item).ok_or_else(|| RuleError::BadDay(item.to_string()))?);
            }
        }
        Ok(())
    }

    pub(super) fn refuse_mixed_day_kinds(&self) -> Result<(), RuleError> {
        if self.by_nth_weekday.is_empty() {
            return Ok(());
        }
        let nth = self.nth_weekday_list();
        if !self.by_day.is_empty() {
            return Err(RuleError::WeekdaysMixedWithNth {
                plain: self.weekday_list(),
                nth,
            });
        }
        if !self.by_month_day.is_empty() {
            return Err(RuleError::MonthDaysMixedWithNth {
                plain: self.month_day_list(),
                nth,
            });
        }
        Ok(())
    }

    pub(super) fn nth_weekday_list(&self) -> String {
        let days: Vec<String> = self.by_nth_weekday.iter().map(NthWeekday::text).collect();
        days.join(",")
    }
}
