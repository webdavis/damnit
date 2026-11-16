use crate::nvim_fixtures::{answer, assert_fixture, error_document};
use crate::support;

use support::fixture::{self, STANDUP};
use support::sandbox::Sandbox;
use support::tokens::{grant, grant_to};

const EXIT_REFUSED: i32 = 4;

fn planted(sb: &Sandbox) -> String {
    sb.new_object(&[
        "plant the tomatoes",
        "--path",
        "home/",
        "--due",
        "2026-09-25",
        "--deadline",
        "2026-09-30",
        "--priority",
        "2",
        "--labels",
        "garden",
        "--category",
        "effort=deep",
        "--estimate",
        "45",
        "--recurrence",
        "every week",
        "--body",
        "loosen the soil\nwater after",
    ])
}

fn edited(sb: &Sandbox, oid: &str, flags: &[&str]) -> String {
    let args = [&["edit", oid], flags, &["-o", "json"]].concat();
    answer(sb, &args, "edit.schema.json")
}

#[test]
fn the_show_fixtures_are_what_dam_show_answers_for_a_task_and_an_event() {
    let _guard =
        support::guard("the_show_fixtures_are_what_dam_show_answers_for_a_task_and_an_event");
    let sb = fixture::seeded();
    let task = planted(&sb);
    let shown = answer(&sb, &["show", &task, "-o", "json"], "show.schema.json");
    assert_fixture("show.json", &shown);
    let standup = fixture::oid(STANDUP);
    edited(&sb, &standup, &["--location", "room 4"]);
    let event = answer(&sb, &["show", &standup, "-o", "json"], "show.schema.json");
    assert_fixture("show-event.json", &event);
}

#[test]
fn the_edit_fixture_is_what_dam_edit_answers() {
    let _guard = support::guard("the_edit_fixture_is_what_dam_edit_answers");
    let sb = fixture::seeded();
    let task = planted(&sb);
    assert_fixture("edit.json", &edited(&sb, &task, &["--due", "2026-10-01"]));
}

#[test]
fn every_flag_the_task_buffer_sends_is_one_dam_edit_takes() {
    let _guard = support::guard("every_flag_the_task_buffer_sends_is_one_dam_edit_takes");
    let sb = fixture::seeded();
    let task = planted(&sb);
    for flags in [
        &["--subject", "plant the peppers"][..],
        &["--body", "only in the morning"],
        &["--due", "2026-10-02"],
        &["--no-due"],
        &["--deadline", "2026-10-09"],
        &["--no-deadline"],
        &["--priority", "1"],
        &["--labels", "seeds", "--labels", "spring"],
        &["--unlabels", "garden"],
        &["--category", "effort=light", "--unlabels", "deep"],
        &["--category", "effort=none"],
        &["--estimate", "1h"],
        &["--no-estimate"],
        &["--recurrence", "every 2 weeks"],
        &["--no-recurrence"],
    ] {
        edited(&sb, &task, flags);
    }
    let standup = fixture::oid(STANDUP);
    for flags in [
        &["--start", "2026-09-25T10:00"][..],
        &["--end", "2026-09-25T10:30"],
        &["--location", "room 4"],
        &["--no-location"],
    ] {
        edited(&sb, &standup, flags);
    }
}

#[test]
fn the_refused_edit_fixture_is_what_dam_prints_for_a_second_exclusive_label() {
    let _guard =
        support::guard("the_refused_edit_fixture_is_what_dam_prints_for_a_second_exclusive_label");
    let sb = fixture::seeded();
    let task = planted(&sb);
    let refused = sb.output(&["edit", &task, "--category", "effort=light", "-o", "json"]);
    assert_fixture(
        "error-refused-edit.json",
        &error_document(refused, EXIT_REFUSED),
    );
}

#[test]
fn the_sent_fixture_is_what_a_tokens_edit_answers() {
    let _guard = support::guard("the_sent_fixture_is_what_a_tokens_edit_answers");
    let sb = fixture::seeded();
    let task = planted(&sb);
    let held = grant_to(&sb, "tool", 1, vec![grant(&["task:read", "task:change"])]);
    let out = sb.output_holding(&held, &["edit", &task, "--due", "2026-10-01", "-o", "json"]);
    assert!(out.status.success(), "{out:?}");
    let text = String::from_utf8(out.stdout).unwrap();
    support::schema::assert_matches_schema(
        &serde_json::from_str(&text).unwrap(),
        "sent.schema.json",
    );
    assert_fixture("sent.json", &text);
}
