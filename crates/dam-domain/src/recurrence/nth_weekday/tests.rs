use crate::recurrence::{Rule, roll_forward};
use jiff::civil::date;

fn next(rule: &str, from: jiff::civil::Date) -> Option<jiff::civil::Date> {
    Rule::parse(rule).unwrap().next_after(from)
}

#[test]
fn every_month_on_1tue_lands_on_the_first_tuesday_of_the_next_month() {
    assert_eq!(
        next("every month on 1tue", date(2026, 11, 3)),
        Some(date(2026, 12, 1))
    );
    assert_eq!(
        next("every month on 1tue", date(2026, 11, 2)),
        Some(date(2026, 11, 3))
    );
}

#[test]
fn every_month_on_3thu_crosses_the_year_end() {
    assert_eq!(
        next("every month on 3thu", date(2026, 12, 17)),
        Some(date(2027, 1, 21))
    );
}

#[test]
fn every_month_on_minus_1fri_lands_on_the_last_friday() {
    assert_eq!(
        next("every month on -1fri", date(2026, 11, 27)),
        Some(date(2026, 12, 25))
    );
    assert_eq!(
        next("every month on -1fri", date(2024, 2, 1)),
        Some(date(2024, 2, 23))
    );
}

#[test]
fn a_leap_february_holds_a_fifth_thursday_and_others_are_skipped() {
    assert_eq!(
        next("every month on 5thu", date(2024, 2, 1)),
        Some(date(2024, 2, 29))
    );
    assert_eq!(
        next("every month on 5thu", date(2026, 1, 29)),
        Some(date(2026, 4, 30))
    );
}

#[test]
fn every_2_months_on_1mon_and_3mon_takes_both_then_skips_a_month() {
    let rule = "every 2 months on 1mon,3mon";
    assert_eq!(next(rule, date(2026, 11, 2)), Some(date(2026, 11, 16)));
    assert_eq!(next(rule, date(2026, 11, 16)), Some(date(2027, 1, 4)));
}

#[test]
fn until_stops_an_ordinal_weekday_rule() {
    let rule = "every month on 1tue until 2026-12-31";
    assert_eq!(next(rule, date(2026, 11, 3)), Some(date(2026, 12, 1)));
    assert_eq!(next(rule, date(2026, 12, 1)), None);
}

#[test]
fn from_completion_counts_from_the_day_it_was_done() {
    let rule = Rule::parse("every! month on -1fri").unwrap();
    assert_eq!(
        roll_forward(&rule, date(2026, 10, 30), date(2026, 12, 2)),
        Some(date(2026, 12, 25))
    );
}

#[test]
fn the_text_form_round_trips() {
    for text in [
        "every month on 1tue",
        "every month on -1fri",
        "every 2 months on 1mon,3mon",
        "every! month on 5sun until 2027-06-30",
    ] {
        assert_eq!(Rule::parse(text).unwrap().to_text(), text);
    }
    assert_eq!(
        Rule::parse("every month on 3MON,1Tuesday,1tue")
            .unwrap()
            .to_text(),
        "every month on 1tue,3mon"
    );
}

#[test]
fn an_ordinal_weekday_outside_a_monthly_rule_is_refused_with_the_monthly_rule() {
    for unit in ["day", "week", "year"] {
        assert_eq!(
            Rule::parse(&format!("every {unit} on 1tue"))
                .unwrap_err()
                .to_string(),
            "on 1tue names a weekday of the month, which only a monthly rule takes; \
             try every month on 1tue"
        );
    }
}

#[test]
fn an_ordinal_past_five_either_way_is_refused_with_the_range() {
    for item in ["0tue", "6tue", "-6fri", "-0fri"] {
        assert_eq!(
            Rule::parse(&format!("every month on {item}"))
                .unwrap_err()
                .to_string(),
            format!(
                "{item:?} counts a weekday past the month; count 1 to 5 from its start \
                 or -1 to -5 from its end; try every month on 1tue or every month on -1fri"
            )
        );
    }
}

#[test]
fn a_mix_of_day_kinds_in_one_rule_is_refused_with_one_kind_to_type() {
    assert_eq!(
        Rule::parse("every month on mon,1tue")
            .unwrap_err()
            .to_string(),
        "on mon,1tue mixes weekdays with weekdays of the month, which one rule cannot hold; \
         try every week on mon or every month on 1tue"
    );
    assert_eq!(
        Rule::parse("every month on 15,1tue")
            .unwrap_err()
            .to_string(),
        "on 15,1tue mixes days of the month with weekdays of the month, which one rule cannot hold; \
         try every month on 15 or every month on 1tue"
    );
}

#[test]
fn the_unknown_day_message_names_the_weekday_of_the_month() {
    let said = Rule::parse("every month on 1funday")
        .unwrap_err()
        .to_string();
    for form in ["1tue", "-1fri"] {
        assert!(said.contains(form), "{said}");
    }
}
