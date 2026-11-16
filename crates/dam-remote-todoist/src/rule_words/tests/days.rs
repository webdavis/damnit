use super::super::{from_todoist, to_todoist};

#[test]
fn weekdays_every_week_are_sent_in_todoists_words() {
    for (dam, todoist) in [
        ("every week on mon,fri", "every mon, fri"),
        ("every week on fri", "every fri"),
        ("every! week on Monday,Friday", "every! Monday, Friday"),
        (
            "every week on mon,fri until 2027-06-30",
            "every mon, fri until 2027-06-30",
        ),
    ] {
        assert_eq!(to_todoist(dam), Ok(todoist.to_string()), "{dam}");
    }
}

#[test]
fn days_of_every_month_are_sent_in_todoists_words() {
    for (dam, todoist) in [
        ("every month on 1,15", "every 1, 15"),
        ("every month on 2,15,27", "every 2, 15, 27"),
        ("every month on 31", "every 31"),
        (
            "every! month on 1,15 until 2027-01-31",
            "every! 1, 15 until 2027-01-31",
        ),
    ] {
        assert_eq!(to_todoist(dam), Ok(todoist.to_string()), "{dam}");
    }
}

#[test]
fn one_weekday_every_two_weeks_is_sent_as_every_other() {
    for (dam, todoist) in [
        ("every 2 weeks on fri", "every other fri"),
        ("every! 2 weeks on Friday", "every! other Friday"),
        (
            "every 2 weeks on fri until 2027-06-30",
            "every other fri until 2027-06-30",
        ),
    ] {
        assert_eq!(to_todoist(dam), Ok(todoist.to_string()), "{dam}");
    }
}

#[test]
fn every_other_shape_is_sent_as_dam_writes_it() {
    for rule in [
        "every 2 weeks on mon,fri",
        "every 3 weeks on fri",
        "every 2 months on 1,15",
        "every day",
        "every 3 days",
        "every week",
        "every 2 weeks",
        "every month",
        "every! 2 months",
        "every year",
        "every 2 years",
        "every day until 2026-12-31",
    ] {
        assert_eq!(to_todoist(rule), Ok(rule.to_string()), "{rule}");
    }
}

#[test]
fn todoists_weekday_and_day_lists_read_as_the_dam_rule() {
    for (todoist, dam) in [
        ("every mon, fri", "every week on mon,fri"),
        ("every Monday, Friday", "every week on Monday,Friday"),
        ("every FRI", "every week on FRI"),
        ("every 2, 15, 27", "every month on 2,15,27"),
        (
            "every! 1, 15 until 2027-01-31",
            "every! month on 1,15 until 2027-01-31",
        ),
    ] {
        assert_eq!(from_todoist(todoist), dam, "{todoist}");
    }
}

#[test]
fn todoists_every_other_reads_as_an_interval_of_two() {
    for (todoist, dam) in [
        ("every other fri", "every 2 weeks on fri"),
        ("every other Friday", "every 2 weeks on Friday"),
        (
            "every! other fri until 2027-06-30",
            "every! 2 weeks on fri until 2027-06-30",
        ),
        ("every other day", "every 2 days"),
        ("every other week", "every 2 weeks"),
        ("every other month", "every 2 months"),
        ("every Other Year", "every 2 years"),
    ] {
        assert_eq!(from_todoist(todoist), dam, "{todoist}");
    }
}

#[test]
fn todoist_text_near_these_forms_stays_its_own() {
    for text in [
        "every other mon, fri",
        "every 0, 15",
        "every 32",
        "every mon, 15",
        "every other",
        "every 3 days",
        "every workday",
    ] {
        assert_eq!(from_todoist(text), text, "{text}");
    }
}
