use std::fmt;

use jiff::civil::Weekday;

use super::{Anchor, Freq, Rule, WIDEST_WEEKLY_INTERVAL_WITH_WEEKDAYS};
use crate::weekday::{parse_weekday, weekday_text};

#[derive(Debug, PartialEq, Eq)]
pub enum RuleError {
    Empty,
    Unknown(String),
    BadInterval(String),
    IntervalTooLarge {
        got: String,
        unit: &'static str,
        max: u32,
    },
    BadDay(String),
    BadDate(String),
    WeekdaysWantWeekly(String),
    MonthDaysWantMonthly(String),
}

impl Rule {
    pub fn parse(text: &str) -> Result<Rule, RuleError> {
        let mut words = text.split_whitespace().peekable();
        let head = words.next().ok_or(RuleError::Empty)?;
        let anchor = match head {
            "every" => Anchor::Due,
            "every!" => Anchor::Completion,
            other => return Err(RuleError::Unknown(other.to_string())),
        };
        let mut interval = 1u32;
        if let Some(n) = words
            .peek()
            .and_then(|w| w.parse::<u32>().ok().map(|n| (n, *w)))
        {
            if n.0 == 0 {
                return Err(RuleError::BadInterval(n.1.to_string()));
            }
            interval = n.0;
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
        if interval > freq.widest_steppable_interval() {
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
        if !rule.by_day.is_empty() && interval > WIDEST_WEEKLY_INTERVAL_WITH_WEEKDAYS {
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
        if self.interval != 1 {
            out.push_str(&format!(" {}", self.interval));
        }
        out.push(' ');
        out.push_str(self.freq.unit_text(self.interval != 1));
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

impl fmt::Display for RuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuleError::Empty => f.write_str("a recurrence rule cannot be empty"),
            RuleError::Unknown(w) => write!(f, "unknown word in recurrence rule: {w:?}"),
            RuleError::BadInterval(w) => write!(f, "interval must be 1 or more, got {w:?}"),
            RuleError::IntervalTooLarge { got, unit, max } => {
                write!(f, "interval must be 1 to {max} {unit}, got {got:?}")
            }
            RuleError::BadDay(w) => write!(
                f,
                "unknown day {w:?}; use a weekday by short or full name in any case, \
                 such as mon or Monday, or a day of the month from 1 to 31"
            ),
            RuleError::BadDate(w) => write!(f, "until wants YYYY-MM-DD, got {w:?}"),
            RuleError::WeekdaysWantWeekly(days) => write!(
                f,
                "on {days} names weekdays, which only a weekly rule takes; try every week on {days}"
            ),
            RuleError::MonthDaysWantMonthly(days) => write!(
                f,
                "on {days} names days of the month, which only a monthly rule takes; try every month on {days}"
            ),
        }
    }
}

impl std::error::Error for RuleError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_interval_past_its_units_calendar_span_is_refused_when_read_not_when_used() {
        for (unit, max) in [
            ("days", 7_304_484u32),
            ("weeks", 1_043_497),
            ("months", 239_976),
            ("years", 19_998),
        ] {
            assert!(
                Rule::parse(&format!("every {max} {unit}")).is_ok(),
                "{max} {unit} is the widest the calendar holds"
            );
            assert_eq!(
                Rule::parse(&format!("every {} {unit}", max + 1)),
                Err(RuleError::IntervalTooLarge {
                    got: (max + 1).to_string(),
                    unit,
                    max,
                })
            );
        }
    }

    #[test]
    fn the_too_large_interval_message_names_a_range_and_unit_that_parse() {
        let said = Rule::parse("every 2147483647 weeks")
            .unwrap_err()
            .to_string();
        for part in ["1 to 1043497", "weeks", "2147483647"] {
            assert!(said.contains(part), "{said}");
        }
        assert!(Rule::parse("every 1043497 weeks").is_ok());
    }

    #[test]
    fn an_on_list_the_frequency_does_not_use_is_refused_with_the_rule_that_takes_it() {
        assert_eq!(
            Rule::parse("every day on mon").unwrap_err().to_string(),
            "on mon names weekdays, which only a weekly rule takes; try every week on mon"
        );
        assert_eq!(
            Rule::parse("every week on 5").unwrap_err().to_string(),
            "on 5 names days of the month, which only a monthly rule takes; try every month on 5"
        );
        assert_eq!(
            Rule::parse("every year on 1,15").unwrap_err().to_string(),
            "on 1,15 names days of the month, which only a monthly rule takes; try every month on 1,15"
        );
        assert!(Rule::parse("every week on mon").is_ok());
        assert!(Rule::parse("every month on 5").is_ok());
    }

    #[test]
    fn a_weekly_interval_the_weekday_scan_cannot_reach_is_refused() {
        assert_eq!(
            Rule::parse("every 210 weeks on mon")
                .unwrap_err()
                .to_string(),
            "interval must be 1 to 208 weeks when weekdays are named, got \"210\""
        );
        assert_eq!(
            Rule::parse("every 209 weeks on mon"),
            Err(RuleError::IntervalTooLarge {
                got: "209".into(),
                unit: "weeks when weekdays are named",
                max: 208,
            })
        );
        assert!(Rule::parse("every 208 weeks on mon").is_ok());
        assert!(Rule::parse("every 210 weeks").is_ok());
    }

    #[test]
    fn the_unknown_day_message_names_forms_the_parser_accepts() {
        let said = RuleError::BadDay("funday".into()).to_string();
        for form in ["mon", "Monday", "any case", "1 to 31"] {
            assert!(said.contains(form), "{said}");
        }
        for text in ["mon", "Monday"] {
            assert!(
                Rule::parse(&format!("every week on {text}")).is_ok(),
                "the message names {text:?} and the parser refuses it"
            );
        }
        assert!(Rule::parse("every month on 31").is_ok());
        assert!(Rule::parse("every month on 32").is_err());
    }
}
