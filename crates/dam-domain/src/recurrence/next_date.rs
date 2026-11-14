use jiff::Span;

use super::{Anchor, Freq, Rule, WIDEST_WEEKLY_INTERVAL_WITH_WEEKDAYS, span};
use crate::Date;
use crate::weekday::week_start;

const MAX_DAYS_SCANNED_BEFORE_GIVING_UP: i32 =
    (WIDEST_WEEKLY_INTERVAL_WITH_WEEKDAYS as i32 + 1) * 7;

impl Rule {
    pub fn next_after(&self, date: Date) -> Option<Date> {
        let candidate = match self.freq {
            Freq::Weekly if !self.by_day.is_empty() => self.next_weekly_on(date),
            Freq::Monthly if !self.by_month_day.is_empty() => self.next_monthly_on(date),
            _ => date.checked_add(self.step()?).ok(),
        }?;
        match self.until {
            Some(until) if candidate > until => None,
            _ => Some(candidate),
        }
    }

    fn step(&self) -> Option<Span> {
        span(self.freq, self.interval.get())
    }

    fn next_weekly_on(&self, from: Date) -> Option<Date> {
        let anchor_week = week_start(from);
        let interval = i64::from(self.interval.get());
        let mut day = from;
        for _ in 0..MAX_DAYS_SCANNED_BEFORE_GIVING_UP {
            day = day.tomorrow().ok()?;
            let weeks_between = i64::from((week_start(day) - anchor_week).get_days() / 7);
            if self.by_day.contains(&day.weekday()) && weeks_between.rem_euclid(interval) == 0 {
                return Some(day);
            }
        }
        None
    }

    fn next_monthly_on(&self, from: Date) -> Option<Date> {
        let step = self.step()?;
        let mut month_start = from.first_of_month();
        for _ in 0..48 {
            let days = if month_start == from.first_of_month() {
                self.by_month_day
                    .iter()
                    .filter(|d| **d > from.day())
                    .copied()
                    .collect::<Vec<_>>()
            } else {
                self.by_month_day.clone()
            };
            for day in days {
                if let Ok(candidate) =
                    jiff::civil::Date::new(month_start.year(), month_start.month(), day)
                {
                    return Some(candidate);
                }
            }
            month_start = month_start.checked_add(step).ok()?;
        }
        None
    }
}

pub fn roll_forward(rule: &Rule, due: Date, completed_on: Date) -> Option<Date> {
    match rule.anchor {
        Anchor::Due => rule.next_after(due),
        Anchor::Completion => rule.next_after(completed_on),
    }
}
