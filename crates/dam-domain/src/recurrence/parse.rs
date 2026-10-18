use std::num::NonZeroU32;

use jiff::civil::Weekday;

use super::{Anchor, Freq, Rule, RuleError, WIDEST_WEEKLY_INTERVAL_WITH_WEEKDAYS};
use crate::weekday::{parse_weekday, weekday_text};

impl Rule {
    pub fn parse(text: &str) -> Result<Rule, RuleError> {
        let mut words = text.split_whitespace().peekable();
        let head = words.next().ok_or(RuleError::Empty)?;
        let anchor = match head {
            "every" => Anchor::Due,
            "every!" => Anchor::Completion,
            other => return Err(RuleError::Unknown(other.to_string())),
        };
        let mut interval = NonZeroU32::MIN;
        if let Some((n, word)) = words
            .peek()
            .and_then(|w| w.parse::<u32>().ok().map(|n| (n, *w)))
        {
            interval =
                NonZeroU32::new(n).ok_or_else(|| RuleError::BadInterval(word.to_string()))?;
            words.next();
        }
        let unit = words.next().ok_or(RuleError::Empty)?;
        let freq = match unit {
            "day" | "days" => Freq::Daily,
            "week" | "weeks" => Freq::Weekly,
            "month" | "months" => Freq::Monthly,
            "year" | "years" => Freq::Yearly,
            other => return Err(RuleError::Unknown(other.to_string())),
        };
        if interval.get() > freq.widest_steppable_interval() {
            return Err(RuleError::IntervalTooLarge {
                got: interval.to_string(),
                unit: freq.unit_text(true),
                max: freq.widest_steppable_interval(),
            });
        }
        let mut rule = Rule {
            freq,
            interval,
            by_day: vec![],
            by_month_day: vec![],
            until: None,
            anchor,
        };
        while let Some(word) = words.next() {
            match word {
                "on" => {
                    let list = words.next().ok_or(RuleError::Empty)?;
                    for item in list.split(',') {
                        if let Ok(day) = item.parse::<i8>() {
                            if !(1..=31).contains(&day) {
                                return Err(RuleError::BadDay(item.to_string()));
                            }
                            rule.by_month_day.push(day);
                        } else {
                            rule.by_day.push(weekday(item)?);
                        }
                    }
                }
                "until" => {
                    let raw = words.next().ok_or(RuleError::Empty)?;
                    rule.until = Some(
                        raw.parse()
                            .map_err(|_| RuleError::BadDate(raw.to_string()))?,
                    );
                }
                other => return Err(RuleError::Unknown(other.to_string())),
            }
        }
        rule.by_day.sort_by_key(|d| d.to_monday_zero_offset());
        rule.by_day.dedup();
        rule.by_month_day.sort_unstable();
        rule.by_month_day.dedup();
        if !rule.by_day.is_empty() && freq != Freq::Weekly {
            return Err(RuleError::WeekdaysWantWeekly(rule.weekday_list()));
        }
        if !rule.by_month_day.is_empty() && freq != Freq::Monthly {
            return Err(RuleError::MonthDaysWantMonthly(rule.month_day_list()));
        }
        if !rule.by_day.is_empty() && interval.get() > WIDEST_WEEKLY_INTERVAL_WITH_WEEKDAYS {
            return Err(RuleError::IntervalTooLarge {
                got: interval.to_string(),
                unit: "weeks when weekdays are named",
                max: WIDEST_WEEKLY_INTERVAL_WITH_WEEKDAYS,
            });
        }
        Ok(rule)
    }

    pub fn to_text(&self) -> String {
        let mut out = String::from(match self.anchor {
            Anchor::Due => "every",
            Anchor::Completion => "every!",
        });
        if self.interval.get() != 1 {
            out.push_str(&format!(" {}", self.interval));
        }
        out.push(' ');
        out.push_str(self.freq.unit_text(self.interval.get() != 1));
        if !self.by_day.is_empty() {
            out.push_str(&format!(" on {}", self.weekday_list()));
        }
        if !self.by_month_day.is_empty() {
            out.push_str(&format!(" on {}", self.month_day_list()));
        }
        if let Some(until) = self.until {
            out.push_str(&format!(" until {until}"));
        }
        out
    }

    fn weekday_list(&self) -> String {
        let days: Vec<&str> = self.by_day.iter().map(|d| weekday_text(*d)).collect();
        days.join(",")
    }

    fn month_day_list(&self) -> String {
        let days: Vec<String> = self.by_month_day.iter().map(|d| d.to_string()).collect();
        days.join(",")
    }
}

fn weekday(text: &str) -> Result<Weekday, RuleError> {
    parse_weekday(text).ok_or_else(|| RuleError::BadDay(text.to_string()))
}

#[cfg(test)]
mod tests;
