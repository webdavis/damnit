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
