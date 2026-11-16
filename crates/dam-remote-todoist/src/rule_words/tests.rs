use super::{Unheld, from_todoist, to_todoist};

mod days;

#[test]
fn a_weekday_of_the_month_is_sent_in_todoists_words() {
    for (dam, todoist) in [
        ("every month on 1tue", "every 1st tue"),
        ("every month on 2tue", "every 2nd tue"),
        ("every month on 3thu", "every 3rd thu"),
        ("every month on 4mon", "every 4th mon"),
        ("every month on 5mon", "every 5th mon"),
        ("every month on -1fri", "every last fri"),
        ("every month on 1mon,3mon", "every 1st mon, 3rd mon"),
        ("every! month on 1Tuesday", "every! 1st Tuesday"),
        (
            "every month on -1fri until 2027-06-30",
            "every last fri until 2027-06-30",
        ),
    ] {
        assert_eq!(to_todoist(dam), Ok(todoist.to_string()), "{dam}");
    }
}

#[test]
fn a_rule_without_a_weekday_of_the_month_is_sent_as_written() {
    for rule in [
        "every 2 weeks on mon,fri",
        "every 3 weeks on fri",
        "every day until 2026-12-31",
    ] {
        assert_eq!(to_todoist(rule), Ok(rule.to_string()));
    }
}

#[test]
fn todoist_holds_no_monthly_interval_with_a_weekday_of_the_month() {
    assert_eq!(
        to_todoist("every 2 months on 1mon,3mon"),
        Err(Unheld::Interval {
            days: "1mon,3mon".into()
        })
    );
}

#[test]
fn todoist_counts_back_from_the_month_end_only_to_the_last_weekday() {
    assert_eq!(
        to_todoist("every month on -2fri"),
        Err(Unheld::CountedBack {
            weekday: "fri".into()
        })
    );
}

#[test]
fn every_rule_dam_sends_reads_back_as_the_same_rule() {
    for rule in [
        "every month on 1tue",
        "every month on 2tue",
        "every month on 3thu",
        "every month on 4mon",
        "every month on 5mon",
        "every month on -1fri",
        "every month on 1mon,3mon",
        "every month on -1fri,1mon",
        "every! month on 1Tuesday until 2027-01-31",
        "every week on mon,fri",
        "every week on fri",
        "every! week on Monday,Friday until 2027-01-31",
        "every month on 1,15",
        "every month on 2,15,27",
        "every! month on 31 until 2027-01-31",
        "every 2 weeks on fri",
        "every! 2 weeks on Friday until 2027-06-30",
        "every 2 weeks on mon,fri",
        "every 3 weeks on fri",
        "every 2 months on 1,15",
        "every 3 days",
        "every year",
    ] {
        assert_eq!(from_todoist(&to_todoist(rule).unwrap()), rule);
    }
}

#[test]
fn todoists_ordinal_words_read_as_a_weekday_of_the_month() {
    assert_eq!(from_todoist("every first wed"), "every month on 1wed");
    assert_eq!(from_todoist("every fifth Friday"), "every month on 5Friday");
}

#[test]
fn any_other_todoist_text_stays_its_own() {
    for text in [
        "every 3rd Tuesday starting Aug 29",
        "every last workday",
        "every day",
        "every 1st wed jan, 3rd thu jul",
        "every 1st",
    ] {
        assert_eq!(from_todoist(text), text);
    }
}
