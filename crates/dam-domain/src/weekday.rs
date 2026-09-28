use jiff::civil::Weekday;

pub fn parse_weekday(text: &str) -> Option<Weekday> {
    match text.to_ascii_lowercase().as_str() {
        "mon" | "monday" => Some(Weekday::Monday),
        "tue" | "tuesday" => Some(Weekday::Tuesday),
        "wed" | "wednesday" => Some(Weekday::Wednesday),
        "thu" | "thursday" => Some(Weekday::Thursday),
        "fri" | "friday" => Some(Weekday::Friday),
        "sat" | "saturday" => Some(Weekday::Saturday),
        "sun" | "sunday" => Some(Weekday::Sunday),
        _ => None,
    }
}

pub fn weekday_text(day: Weekday) -> &'static str {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_day_reads_by_short_name_full_name_and_any_case() {
        for text in ["thu", "thursday", "Thursday", "THU"] {
            assert_eq!(parse_weekday(text), Some(Weekday::Thursday), "{text}");
        }
    }

    #[test]
    fn a_word_that_names_no_day_is_none() {
        for text in ["", "thur", "tues", "someday", "1"] {
            assert_eq!(parse_weekday(text), None, "{text}");
        }
    }

    #[test]
    fn every_day_writes_back_as_a_name_that_reads_again() {
        for day in [
            Weekday::Monday,
            Weekday::Tuesday,
            Weekday::Wednesday,
            Weekday::Thursday,
            Weekday::Friday,
            Weekday::Saturday,
            Weekday::Sunday,
        ] {
            assert_eq!(parse_weekday(weekday_text(day)), Some(day));
        }
    }

    #[test]
    fn a_weekday_name_reads_the_same_in_a_recurrence_rule_and_a_date_word() {
        let friday = jiff::civil::date(2026, 9, 18);
        let tz = jiff::tz::TimeZone::UTC;
        for text in ["mon", "monday", "Monday", "MON"] {
            let rule = crate::Rule::parse(&format!("every week on {text}")).unwrap();
            assert_eq!(rule.by_day, vec![Weekday::Monday], "{text}");
            let when = crate::When::parse_human(text, friday, &tz).unwrap();
            assert_eq!(when.date().weekday(), Weekday::Monday, "{text}");
        }
    }
}
