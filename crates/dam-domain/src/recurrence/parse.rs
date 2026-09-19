use std::fmt;

use jiff::civil::Weekday;

use super::{Anchor, Freq, Rule};

#[derive(Debug, PartialEq, Eq)]
pub enum RuleError {
    Empty,
    Unknown(String),
    BadInterval(String),
    BadDay(String),
    BadDate(String),
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
        let plural = self.interval != 1;
        out.push(' ');
        out.push_str(match (self.freq, plural) {
            (Freq::Daily, false) => "day",
            (Freq::Daily, true) => "days",
            (Freq::Weekly, false) => "week",
            (Freq::Weekly, true) => "weeks",
            (Freq::Monthly, false) => "month",
            (Freq::Monthly, true) => "months",
            (Freq::Yearly, false) => "year",
            (Freq::Yearly, true) => "years",
        });
        if !self.by_day.is_empty() {
            let days: Vec<&str> = self.by_day.iter().map(|d| weekday_text(*d)).collect();
            out.push_str(&format!(" on {}", days.join(",")));
        }
        if !self.by_month_day.is_empty() {
            let days: Vec<String> = self.by_month_day.iter().map(|d| d.to_string()).collect();
            out.push_str(&format!(" on {}", days.join(",")));
        }
        if let Some(until) = self.until {
            out.push_str(&format!(" until {until}"));
        }
        out
    }
}

fn weekday(text: &str) -> Result<Weekday, RuleError> {
    Ok(match text {
        "mon" => Weekday::Monday,
        "tue" => Weekday::Tuesday,
        "wed" => Weekday::Wednesday,
        "thu" => Weekday::Thursday,
        "fri" => Weekday::Friday,
        "sat" => Weekday::Saturday,
        "sun" => Weekday::Sunday,
        other => return Err(RuleError::BadDay(other.to_string())),
    })
}

fn weekday_text(day: Weekday) -> &'static str {
    match day {
        Weekday::Monday => "mon",
        Weekday::Tuesday => "tue",
        Weekday::Wednesday => "wed",
        Weekday::Thursday => "thu",
        Weekday::Friday => "fri",
        Weekday::Saturday => "sat",
        Weekday::Sunday => "sun",
    }
}

impl fmt::Display for RuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuleError::Empty => f.write_str("a recurrence rule cannot be empty"),
            RuleError::Unknown(w) => write!(f, "unknown word in recurrence rule: {w:?}"),
            RuleError::BadInterval(w) => write!(f, "interval must be 1 or more, got {w:?}"),
            RuleError::BadDay(w) => write!(
                f,
                "unknown day {w:?}; use mon,tue,wed,thu,fri,sat,sun or a day of the month"
            ),
            RuleError::BadDate(w) => write!(f, "until wants YYYY-MM-DD, got {w:?}"),
        }
    }
}

impl std::error::Error for RuleError {}
