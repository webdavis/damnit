use crate::nvim_fixtures::{assert_fixture, fed_answer};
use crate::support;

use support::fixture;
use support::sandbox::Sandbox;

const SENT: [&str; 5] = ["list", "--request", "-", "-o", "json"];

const SECTIONS: [(&str, &str); 4] = [
    ("today-overdue.json", r#"{"overdue":true}"#),
    ("today-today.json", r#"{"due":"today","overdue":false}"#),
    (
        "today-next.json",
        r#"{"due":"today+1..today+7","overdue":false}"#,
    ),
    ("today-none.json", r#"{"due":"none"}"#),
];

fn day(offset: i64) -> String {
    let today = jiff::Timestamp::now()
        .to_zoned(jiff::tz::TimeZone::UTC)
        .date();
    today
        .checked_add(jiff::Span::new().days(offset))
        .unwrap()
        .to_string()
}

fn without_today(text: &str) -> String {
    text.replace(&day(0), "<today>")
        .replace(&day(3), "<today+3>")
}

fn a_week_of_tasks() -> Sandbox {
    let sb = Sandbox::new();
    sb.new_object(&["renew the passport", "--due", "2026-09-14"]);
    sb.new_object(&["call the bank", "--due", "today", "--labels", "money"]);
    sb.new_object(&["send the invoice", "--due", "today+3", "--estimate", "20"]);
    sb.new_object(&["read the lease"]);
    let parent = sb.new_object(&["plan the move"]);
    sb.new_object(&["pack the books", "--parent", &parent]);
    sb.new_object(&["book the van", "--parent", &parent, "--due", "today+3"]);
    sb
}

#[test]
fn the_today_fixtures_are_what_each_section_of_dam_today_answers() {
    let _guard = support::guard("the_today_fixtures_are_what_each_section_of_dam_today_answers");
    let sb = a_week_of_tasks();
    for (name, request) in SECTIONS {
        assert_fixture(name, &without_today(&fed_answer(&sb, &SENT, request)));
    }
}

#[test]
fn the_by_path_fixture_is_what_a_list_sorted_by_path_answers() {
    let _guard = support::guard("the_by_path_fixture_is_what_a_list_sorted_by_path_answers");
    let sb = fixture::seeded();
    sb.new_object(&["pay rent", "--path", "home/", "--due", "2026-09-30"]);
    sb.new_object(&["fix the gate", "--path", "work/", "--labels", "repair"]);
    sb.new_object(&["file the report", "--path", "work/", "--estimate", "30"]);
    let text = fed_answer(&sb, &SENT, r#"{"sort":"path"}"#);
    assert_fixture("list-by-path.json", &text);
}
