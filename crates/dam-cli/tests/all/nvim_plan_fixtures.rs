use crate::nvim_fixtures::{answer, assert_fixture, error_document, fed_answer};
use crate::support;

use support::request::fed;
use support::sandbox::Sandbox;

const EXIT_REFUSED: i32 = 4;
const SENT: [&str; 5] = ["list", "--request", "-", "-o", "json"];
const UNPLANNED: &str = r#"{"blocked":false,"focus.day":"none","limit":50}"#;

fn day(offset: i64) -> String {
    jiff::Timestamp::now()
        .to_zoned(jiff::tz::TimeZone::UTC)
        .date()
        .checked_add(jiff::Span::new().days(offset))
        .unwrap()
        .to_string()
}

fn without_days(text: &str) -> String {
    (0..8).fold(text.to_string(), |text, n| {
        let word = if n == 0 {
            "<today>".to_string()
        } else {
            format!("<today+{n}>")
        };
        let day = day(n);
        text.replace(&format!("{day}\""), &format!("{word}\""))
            .replace(&format!("{day} "), &format!("{word} "))
    })
}

fn a_planned_week() -> Sandbox {
    let sb = Sandbox::new();
    let lease = sb.new_object(&["read the lease"]);
    sb.new_object(&["sign the lease", "--depends", &lease]);
    sb.new_object(&["pay rent", "--focus", "today"]);
    sb.new_object(&["call the bank", "--focus", "today+1", "--labels", "money"]);
    sb
}

fn focused(day: &str) -> String {
    format!(r#"{{"focus.day":"{day}"}}"#)
}

#[test]
fn the_plan_fixtures_are_what_each_section_of_the_week_plan_answers() {
    let _guard = support::guard("the_plan_fixtures_are_what_each_section_of_the_week_plan_answers");
    let sb = a_planned_week();
    let unplanned = fed_answer(&sb, &SENT, UNPLANNED);
    assert_fixture("plan-unplanned.json", &without_days(&unplanned));
    let today = fed_answer(&sb, &SENT, &focused("today"));
    assert_fixture("plan-today.json", &without_days(&today));
    let tomorrow = fed_answer(&sb, &SENT, &focused("today+1"));
    assert_fixture("plan-today+1.json", &without_days(&tomorrow));
    for n in 2..=6 {
        let later = fed_answer(&sb, &SENT, &focused(&format!("today+{n}")));
        assert_fixture("plan-empty.json", &later);
    }
}

#[test]
fn the_focus_fixtures_are_what_dam_edit_answers_when_a_mark_moves_and_when_a_day_is_full() {
    let _guard = support::guard(
        "the_focus_fixtures_are_what_dam_edit_answers_when_a_mark_moves_and_when_a_day_is_full",
    );
    let sb = a_planned_week();
    let lease = sb.new_object(&["renew the lease"]);
    let moved = answer(
        &sb,
        &["edit", &lease, "--focus", "today", "-o", "json"],
        "edit.schema.json",
    );
    assert_fixture("edit-focus.json", &without_days(&moved));
    let both = fed(
        &sb,
        &[
            "edit", "-", "--focus", "today+2", "--expect", "2", "-o", "json",
        ],
        &format!("{lease}\n{}\n", sb.new_object(&["water the plants"])),
    );
    assert!(both.status.success(), "{both:?}");
    answer(
        &sb,
        &["edit", &lease, "--no-focus", "-o", "json"],
        "edit.schema.json",
    );
    sb.new_object(&["book the van", "--focus", "today"]);
    sb.new_object(&["pack the books", "--focus", "today"]);
    let full = sb.output(&["edit", &lease, "--focus", "today", "-o", "json"]);
    assert_fixture(
        "error-focus-full.json",
        &without_days(&error_document(full, EXIT_REFUSED)),
    );
}
