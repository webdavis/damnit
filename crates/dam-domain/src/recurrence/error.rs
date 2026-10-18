use std::fmt;

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
