use super::super::{from_todoist, with_start};

#[test]
fn the_due_date_is_the_start_written_as_day_month_and_year() {
    for (words, due, sent) in [
        (
            "every other fri",
            "2026-12-04",
            "every other fri starting 4 Dec 2026",
        ),
        ("every day", "2027-01-31", "every day starting 31 Jan 2027"),
        (
            "every! 1st mon, last fri",
            "2026-09-25",
            "every! 1st mon, last fri starting 25 Sep 2026",
        ),
    ] {
        assert_eq!(with_start(words, due), sent, "{words}");
    }
}

#[test]
fn a_due_time_follows_the_days_and_the_start_comes_last() {
    assert_eq!(
        with_start(
            "every mon, fri",
            "2026-12-04T20:00:00-05:00[America/New_York]"
        ),
        "every mon, fri at 20:00 starting 4 Dec 2026"
    );
    assert_eq!(
        with_start(
            "every 1, 15 until 2027-06-30",
            "2026-12-04T07:30:00+00:00[UTC]"
        ),
        "every 1, 15 at 07:30 until 2027-06-30 starting 4 Dec 2026"
    );
}

#[test]
fn a_due_that_is_not_a_date_leaves_the_words_alone() {
    assert_eq!(with_start("every day", "someday"), "every day");
}

#[test]
fn the_start_and_time_dam_wrote_are_read_off_before_the_rule() {
    for (todoist, dam) in [
        (
            "every other fri starting 4 Dec 2026",
            "every 2 weeks on fri",
        ),
        ("every other fri from 4 Dec 2026", "every 2 weeks on fri"),
        (
            "every mon, fri at 20:00 starting 4 Dec 2026",
            "every week on mon,fri",
        ),
        (
            "every 1, 15 at 07:30 until 2027-06-30 starting 4 Dec 2026",
            "every month on 1,15 until 2027-06-30",
        ),
        ("every 3 days starting 1 Jan 2027", "every 3 days"),
    ] {
        assert_eq!(from_todoist(todoist), dam, "{todoist}");
    }
}

#[test]
fn a_start_or_time_in_a_form_dam_does_not_write_stays_todoists_text() {
    for text in [
        "every 3rd Tuesday starting Aug 29",
        "every other fri starting 4 December 2026",
        "every day at 7am",
        "every day starting 32 Dec 2026",
    ] {
        assert_eq!(from_todoist(text), text);
    }
}
