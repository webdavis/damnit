pub type Date = jiff::civil::Date;
pub type Timestamp = jiff::Timestamp;

/// A point on the calendar: a whole day, or an instant in a named zone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum When {
    Day(Date),
    At(jiff::Zoned),
}

impl When {
    pub fn date(&self) -> Date {
        match self {
            When::Day(d) => *d,
            When::At(z) => z.date(),
        }
    }

    pub fn is_all_day(&self) -> bool {
        matches!(self, When::Day(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::date;

    #[test]
    fn a_day_is_all_day_and_its_date_is_itself() {
        let w = When::Day(date(2026, 9, 25));
        assert!(w.is_all_day());
        assert_eq!(w.date(), date(2026, 9, 25));
    }

    #[test]
    fn an_instant_reports_its_local_date() {
        let z = date(2026, 9, 25)
            .at(14, 0, 0, 0)
            .in_tz("America/Denver")
            .unwrap();
        let w = When::At(z);
        assert!(!w.is_all_day());
        assert_eq!(w.date(), date(2026, 9, 25));
    }
}
