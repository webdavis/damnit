use super::*;
use crate::testing::context;
use dam_domain::When;
use jiff::civil::date;

#[test]
fn every_date_flag_reads_the_same_words() {
    let friday_2026_09_18 = context();
    let ctx = &friday_2026_09_18;
    for flag in ["due", "start", "end"] {
        assert_eq!(
            when_flag(ctx, flag, "next mon").unwrap(),
            When::Day(date(2026, 9, 21)),
            "--{flag}"
        );
        assert_eq!(
            when_flag(ctx, flag, "in 2 weeks").unwrap(),
            When::Day(date(2026, 10, 2)),
            "--{flag}"
        );
    }
    assert_eq!(deadline_flag(ctx, "next mon").unwrap(), date(2026, 9, 21));
    assert_eq!(deadline_flag(ctx, "in 2 weeks").unwrap(), date(2026, 10, 2));
    assert_eq!(deadline_flag(ctx, "2026-11-01").unwrap(), date(2026, 11, 1));
}

#[test]
fn a_deadline_already_past_is_read_rather_than_refused() {
    let ctx = context();
    assert_eq!(deadline_flag(&ctx, "2020-01-01").unwrap(), date(2020, 1, 1));
}

#[test]
fn a_value_that_is_not_one_day_or_instant_is_refused_naming_the_forms_to_set() {
    let ctx = context();
    for text in [
        "someday",
        "2026-11",
        "week",
        "today..fri",
        "on:thu",
        "none",
        "not:today",
    ] {
        for error in [
            when_flag(&ctx, "end", text).unwrap_err(),
            day_flag(&ctx, "end", text).unwrap_err(),
        ] {
            assert_eq!(error.exit_code(), 2);
            let said = error.to_string();
            assert!(
                said.contains(&format!("--end cannot be {text:?}")),
                "{said}"
            );
            assert!(said.contains("one day or instant"), "{said}");
            for form in ["2026-11-20", "today+3", "now+2h", "tomorrow", "in 3 days"] {
                assert!(said.contains(form), "{said}");
            }
            assert!(said.contains("--end today"), "{said}");
        }
    }
}

#[test]
fn today_with_a_count_and_now_with_a_length_set_a_day_and_an_instant() {
    let ctx = context();
    assert_eq!(
        when_flag(&ctx, "due", "today+1").unwrap(),
        When::Day(date(2026, 9, 19))
    );
    assert_eq!(deadline_flag(&ctx, "today-1").unwrap(), date(2026, 9, 17));
    assert_eq!(
        day_flag(&ctx, "focus", "today+2").unwrap(),
        date(2026, 9, 20)
    );
    let When::At(at) = when_flag(&ctx, "start", "now+90m").unwrap() else {
        panic!("not an instant")
    };
    assert_eq!(
        at.timestamp(),
        ctx.clock.now() + jiff::SignedDuration::from_mins(90)
    );
}

#[test]
fn a_deadline_is_a_whole_day_so_a_time_given_to_it_is_dropped() {
    let ctx = context();
    assert_eq!(
        deadline_flag(&ctx, "2026-11-01T09:30").unwrap(),
        date(2026, 11, 1)
    );
}
