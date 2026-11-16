use crate::nvim_fixtures::{answer, assert_fixture};
use crate::support;

use support::sandbox::Sandbox;

const WEEKDAYS: [&str; 7] = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];

fn today() -> jiff::civil::Date {
    jiff::Timestamp::now()
        .to_zoned(jiff::tz::TimeZone::UTC)
        .date()
}

fn after(day: jiff::civil::Date, days: i64) -> String {
    day.checked_add(jiff::Span::new().days(days))
        .unwrap()
        .to_string()
}

fn next_monday() -> jiff::civil::Date {
    let today = today();
    let back = i64::from(today.weekday().to_monday_zero_offset());
    today.checked_add(jiff::Span::new().days(7 - back)).unwrap()
}

fn without_next_week(text: &str) -> String {
    let monday = next_monday();
    let pinned: jiff::civil::Date = "2029-12-31".parse().unwrap();
    (-7..7).fold(text.to_string(), |text, n| {
        text.replace(&after(monday, n), &after(pinned, n + 7))
    })
}

fn without_today(text: &str) -> String {
    let weekday = WEEKDAYS[today().weekday().to_monday_zero_offset() as usize];
    text.replace(&after(today(), 0), "<today>").replace(
        &format!("\"weekday\": \"{weekday}\""),
        "\"weekday\": \"<today>\"",
    )
}

fn next_week_of_events_and_tasks() -> Sandbox {
    let sb = Sandbox::new();
    let monday = next_monday();
    let (tue, wed, thu) = (after(monday, 1), after(monday, 2), after(monday, 3));
    let event = ["--kind", "event", "--start"];
    sb.new_object(&[&["offsite"][..], &event, &[&tue, "--end", &wed]].concat());
    let (nine, half_past) = (format!("{wed}T09:00"), format!("{wed}T09:30"));
    sb.new_object(&[&["standup"][..], &event, &[&nine, "--end", &half_past]].concat());
    sb.new_object(&["pay rent", "--due", &tue, "--path", "home/"]);
    sb.new_object(&["call the bank", "--due", &thu, "--labels", "money"]);
    sb
}

fn agenda(sb: &Sandbox, window: &str) -> String {
    let line = [
        "agenda", "--kind", "any", "--from", window, "--to", window, "-o", "json",
    ];
    answer(sb, &line, "agenda.schema.json")
}

#[test]
fn the_agenda_fixtures_are_what_dam_agenda_answers_for_each_window_the_plugin_asks_for() {
    let _guard = support::guard(
        "the_agenda_fixtures_are_what_dam_agenda_answers_for_each_window_the_plugin_asks_for",
    );
    let sb = next_week_of_events_and_tasks();
    assert_fixture(
        "agenda-next-week.json",
        &without_next_week(&agenda(&sb, "week+1")),
    );
    assert_fixture("agenda-week.json", &without_next_week(&agenda(&sb, "week")));
    for window in ["week-1", "today+1", "today-1"] {
        agenda(&sb, window);
    }
}

#[test]
fn the_agenda_day_fixture_is_what_dam_agenda_answers_for_today() {
    let _guard = support::guard("the_agenda_day_fixture_is_what_dam_agenda_answers_for_today");
    let sb = Sandbox::new();
    let today = after(today(), 0);
    let (noon, one) = (format!("{today}T12:00"), format!("{today}T13:00"));
    sb.new_object(&["lunch", "--kind", "event", "--start", &noon, "--end", &one]);
    sb.new_object(&["call the bank", "--due", "today"]);
    assert_fixture("agenda-day.json", &without_today(&agenda(&sb, "today")));
}
