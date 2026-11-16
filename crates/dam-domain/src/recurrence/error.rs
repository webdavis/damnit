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
    NthWeekdaysWantMonthly(String),
    BadOrdinal(String),
    WeekdaysMixedWithNth {
        plain: String,
        nth: String,
    },
    MonthDaysMixedWithNth {
        plain: String,
        nth: String,
    },
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
                 such as mon or Monday, a day of the month from 1 to 31, \
                 or a weekday of the month such as 1tue or -1fri"
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
            RuleError::NthWeekdaysWantMonthly(days) => write!(
                f,
                "on {days} names a weekday of the month, which only a monthly rule takes; \
                 try every month on {days}"
            ),
            RuleError::BadOrdinal(w) => write!(
                f,
                "{w:?} counts a weekday past the month; count 1 to 5 from its start \
                 or -1 to -5 from its end; try every month on 1tue or every month on -1fri"
            ),
            RuleError::WeekdaysMixedWithNth { plain, nth } => write!(
                f,
                "on {plain},{nth} mixes weekdays with weekdays of the month, \
                 which one rule cannot hold; try every week on {plain} or every month on {nth}"
            ),
            RuleError::MonthDaysMixedWithNth { plain, nth } => write!(
                f,
                "on {plain},{nth} mixes days of the month with weekdays of the month, \
                 which one rule cannot hold; try every month on {plain} or every month on {nth}"
            ),
        }
    }
}

impl std::error::Error for RuleError {}
