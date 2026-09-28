use super::*;
use jiff::civil::{Weekday, date};

fn hand_built(freq: Freq, interval: u32, by_day: Vec<Weekday>, by_month_day: Vec<i8>) -> Rule {
    Rule {
        freq,
        interval,
        by_day,
        by_month_day,
        until: None,
        anchor: Anchor::Due,
    }
}

#[test]
fn each_units_maximum_is_the_widest_step_the_calendar_holds() {
    for freq in [Freq::Daily, Freq::Weekly, Freq::Monthly, Freq::Yearly] {
        let max = freq.widest_steppable_interval();
        assert!(span(freq, max).is_some(), "{freq:?} {max}");
        assert!(span(freq, max + 1).is_none(), "{freq:?} {}", max + 1);
    }
}

#[test]
fn the_widest_yearly_interval_steps_from_the_earliest_date_to_the_latest_year() {
    let rule = Rule::parse("every 19998 years").unwrap();
    assert_eq!(rule.next_after(date(-9999, 1, 1)), Some(date(9999, 1, 1)));
}

#[test]
fn a_hand_built_interval_the_parser_would_refuse_cannot_advance() {
    for freq in [Freq::Daily, Freq::Weekly, Freq::Monthly, Freq::Yearly] {
        let rule = hand_built(freq, u32::MAX, vec![], vec![]);
        assert_eq!(rule.next_after(date(2026, 9, 18)), None, "{freq:?}");
    }
    let weekly = hand_built(Freq::Weekly, u32::MAX, vec![Weekday::Monday], vec![]);
    assert_eq!(weekly.next_after(date(2026, 9, 18)), None);
    let monthly = hand_built(Freq::Monthly, u32::MAX, vec![], vec![1]);
    assert_eq!(monthly.next_after(date(2026, 9, 18)), None);
}

#[test]
fn a_zero_interval_cannot_advance_rather_than_dividing_by_zero() {
    for freq in [Freq::Daily, Freq::Weekly, Freq::Monthly, Freq::Yearly] {
        let rule = hand_built(freq, 0, vec![], vec![]);
        assert_eq!(rule.next_after(date(2026, 9, 18)), None, "{freq:?}");
    }
    let weekly = hand_built(Freq::Weekly, 0, vec![Weekday::Monday], vec![]);
    assert_eq!(weekly.next_after(date(2026, 9, 18)), None);
    let monthly = hand_built(Freq::Monthly, 0, vec![], vec![1]);
    assert_eq!(monthly.next_after(date(2026, 9, 18)), None);
}

#[test]
fn every_week_from_a_friday_lands_on_the_next_friday() {
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
    let (friday, monday, wednesday) = (date(2026, 9, 18), date(2026, 9, 21), date(2026, 9, 23));
    assert_eq!(friday.weekday(), Weekday::Friday);
    assert_eq!(rule.next_after(friday), Some(monday));
    assert_eq!(rule.next_after(monday), Some(wednesday));
}

#[test]
fn every_two_weeks_on_monday_skips_a_week() {
    let rule = Rule::parse("every 2 weeks on mon").unwrap();
    let (monday, two_mondays_later) = (date(2026, 9, 21), date(2026, 10, 5));
    assert_eq!(rule.next_after(monday), Some(two_mondays_later));
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

#[test]
fn a_month_day_outside_1_to_31_is_named_and_refused() {
    assert_eq!(
        Rule::parse("every month on 0"),
        Err(RuleError::BadDay("0".into()))
    );
    assert_eq!(
        Rule::parse("every month on -1"),
        Err(RuleError::BadDay("-1".into()))
    );
    assert_eq!(
        Rule::parse("every month on 40"),
        Err(RuleError::BadDay("40".into()))
    );
    assert!(Rule::parse("every month on 31").is_ok());
}
