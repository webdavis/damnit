mod parse;

use jiff::ToSpan;
use jiff::civil::Weekday;

use crate::Date;

pub use parse::RuleError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Freq {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

/// `every week` counts from the due date; `every! week` from the completion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchor {
    Due,
    Completion,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    pub freq: Freq,
    pub interval: u32,
    pub by_day: Vec<Weekday>,
    pub by_month_day: Vec<i8>,
    pub until: Option<Date>,
    pub anchor: Anchor,
}

/// Bounds the forward scan so a rule with no reachable date terminates.
const SCAN_LIMIT_DAYS: i32 = 366 * 4;

impl Rule {
    pub fn next_after(&self, date: Date) -> Option<Date> {
        let candidate = match self.freq {
            Freq::Daily => date.checked_add((self.interval as i32).days()).ok(),
            Freq::Weekly if self.by_day.is_empty() => {
                date.checked_add((self.interval as i32).weeks()).ok()
            }
            Freq::Weekly => self.next_weekly_on(date),
            Freq::Monthly if self.by_month_day.is_empty() => {
                date.checked_add((self.interval as i32).months()).ok()
            }
            Freq::Monthly => self.next_monthly_on(date),
            Freq::Yearly => date.checked_add((self.interval as i32).years()).ok(),
        }?;
        match self.until {
            Some(until) if candidate > until => None,
            _ => Some(candidate),
        }
    }

    fn next_weekly_on(&self, from: Date) -> Option<Date> {
        let anchor_week = week_start(from);
        let mut day = from;
        for _ in 0..SCAN_LIMIT_DAYS {
            day = day.tomorrow().ok()?;
            let weeks_between = (week_start(day) - anchor_week).get_days() / 7;
            if self.by_day.contains(&day.weekday()) && weeks_between % self.interval as i32 == 0 {
                return Some(day);
            }
        }
        None
    }

    fn next_monthly_on(&self, from: Date) -> Option<Date> {
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
            month_start = month_start
                .checked_add((self.interval as i32).months())
                .ok()?;
        }
        None
    }
}

fn week_start(date: Date) -> Date {
    let back = date.weekday().since(Weekday::Monday) as i32;
    date.checked_sub(back.days()).unwrap_or(date)
}

pub fn roll_forward(rule: &Rule, due: Date, completed_on: Date) -> Option<Date> {
    match rule.anchor {
        Anchor::Due => rule.next_after(due),
        Anchor::Completion => rule.next_after(completed_on),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::{Weekday, date};

    #[test]
    fn every_week_from_a_thursday_lands_on_the_next_thursday() {
        let rule = Rule::parse("every week").unwrap();
        assert_eq!(rule.next_after(date(2026, 9, 18)), Some(date(2026, 9, 25)));
    }

    #[test]
    fn every_two_days() {
        let rule = Rule::parse("every 2 days").unwrap();
        assert_eq!(rule.next_after(date(2026, 9, 18)), Some(date(2026, 9, 20)));
    }

    #[test]
    fn weekly_on_listed_days_takes_the_next_listed_day() {
        let rule = Rule::parse("every week on mon,wed").unwrap();
        assert_eq!(rule.by_day, vec![Weekday::Monday, Weekday::Wednesday]);
        // 2026-09-18 is a Friday; next Monday is the 21st
        assert_eq!(rule.next_after(date(2026, 9, 18)), Some(date(2026, 9, 21)));
        // from Monday the 21st, next listed is Wednesday the 23rd
        assert_eq!(rule.next_after(date(2026, 9, 21)), Some(date(2026, 9, 23)));
    }

    #[test]
    fn every_two_weeks_on_monday_skips_a_week() {
        let rule = Rule::parse("every 2 weeks on mon").unwrap();
        // from Monday 2026-09-21, the next is Monday 2026-10-05
        assert_eq!(rule.next_after(date(2026, 9, 21)), Some(date(2026, 10, 5)));
    }

    #[test]
    fn monthly_on_a_day_skips_months_without_it() {
        let rule = Rule::parse("every month on 31").unwrap();
        assert_eq!(rule.next_after(date(2026, 1, 31)), Some(date(2026, 3, 31)));
    }

    #[test]
    fn monthly_without_a_day_keeps_the_anchor_day() {
        let rule = Rule::parse("every month").unwrap();
        assert_eq!(rule.next_after(date(2026, 9, 15)), Some(date(2026, 10, 15)));
    }

    #[test]
    fn yearly() {
        let rule = Rule::parse("every year").unwrap();
        assert_eq!(rule.next_after(date(2026, 2, 28)), Some(date(2027, 2, 28)));
    }

    #[test]
    fn until_stops_it() {
        let rule = Rule::parse("every day until 2026-09-20").unwrap();
        assert_eq!(rule.next_after(date(2026, 9, 19)), Some(date(2026, 9, 20)));
        assert_eq!(rule.next_after(date(2026, 9, 20)), None);
    }

    #[test]
    fn bang_anchors_on_completion() {
        let from_due = Rule::parse("every week").unwrap();
        let from_completion = Rule::parse("every! week").unwrap();
        let due = date(2026, 9, 18);
        let completed = date(2026, 9, 22);
        assert_eq!(
            roll_forward(&from_due, due, completed),
            Some(date(2026, 9, 25))
        );
        assert_eq!(
            roll_forward(&from_completion, due, completed),
            Some(date(2026, 9, 29))
        );
    }

    #[test]
    fn to_text_round_trips() {
        for text in [
            "every day",
            "every! 2 weeks on mon,wed",
            "every month on 1,15 until 2027-01-01",
            "every year",
        ] {
            assert_eq!(Rule::parse(text).unwrap().to_text(), text);
        }
    }

    #[test]
    fn monthly_multiple_days_takes_the_earliest_not_the_first_listed() {
        let rule = Rule::parse("every month on 20,5").unwrap();
        assert_eq!(rule.next_after(date(2026, 9, 1)), Some(date(2026, 9, 5)));
        assert_eq!(rule.to_text(), "every month on 5,20");
    }

    #[test]
    fn roll_forward_past_until_is_none() {
        let rule = Rule::parse("every week until 2026-09-24").unwrap();
        let due = date(2026, 9, 18);
        let completed = date(2026, 9, 18);
        assert_eq!(roll_forward(&rule, due, completed), None);
    }

    #[test]
    fn bad_text_is_named() {
        assert_eq!(Rule::parse(""), Err(RuleError::Empty));
        assert_eq!(
            Rule::parse("weekly"),
            Err(RuleError::Unknown("weekly".into()))
        );
        assert_eq!(
            Rule::parse("every 0 days"),
            Err(RuleError::BadInterval("0".into()))
        );
        assert_eq!(
            Rule::parse("every week on funday"),
            Err(RuleError::BadDay("funday".into()))
        );
        assert_eq!(
            Rule::parse("every day until soon"),
            Err(RuleError::BadDate("soon".into()))
        );
    }
}
