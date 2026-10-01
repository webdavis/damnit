use dam_domain::{Attendee, Event, Object, Oid, ResponseStatus, When};

use super::*;
use crate::testing::context;

fn standup() -> Event {
    let start = "2026-09-18T14:00:00+00:00[UTC]".parse().unwrap();
    let end = "2026-09-18T14:30:00+00:00[UTC]".parse().unwrap();
    let mut e = Event::new(
        Oid::generate(&mut |b: &mut [u8]| b.fill(0x3f)),
        "standup",
        When::At(start),
        When::At(end),
    );
    e.base.labels.insert("work".into());
    e
}

fn args(from: Option<&str>, to: Option<&str>) -> AgendaArgs {
    AgendaArgs {
        query: None,
        from: from.map(str::to_string),
        to: to.map(str::to_string),
        max_age: vec![],
    }
}

#[test]
fn the_agenda_document_keeps_its_contract() {
    let mut ctx = context();
    ctx.store.put(&Object::Event(standup())).unwrap();
    let report = run_agenda(&mut ctx, args(None, None)).unwrap();
    assert_eq!(
        report.data,
        serde_json::json!({"events": [{
            "oid": "3f".repeat(20), "subject": "standup", "path": "", "labels": ["work"],
            "status": "confirmed", "transparency": "busy", "all_day": false,
            "start": 1_789_740_000, "end": 1_789_741_800, "busy": true
        }]})
    );
}

fn timed(fill: u8, subject: &str, start: &str, end: &str) -> Object {
    Object::Event(Event::new(
        Oid::generate(&mut |b: &mut [u8]| b.fill(fill)),
        subject,
        When::At(start.parse().unwrap()),
        When::At(end.parse().unwrap()),
    ))
}

fn subjects(report: &Report) -> Vec<&str> {
    let events = report.data["events"].as_array().unwrap();
    events
        .iter()
        .map(|e| e["subject"].as_str().unwrap())
        .collect()
}

#[test]
fn with_no_flags_the_window_is_now_to_a_day_later() {
    let mut ctx = context();
    for event in [
        timed(
            1,
            "early",
            "2026-09-18T00:00:00+00:00[UTC]",
            "2026-09-18T00:30:00+00:00[UTC]",
        ),
        timed(
            2,
            "late",
            "2026-09-18T23:30:00+00:00[UTC]",
            "2026-09-19T00:00:00+00:00[UTC]",
        ),
        timed(
            3,
            "next",
            "2026-09-19T00:00:00+00:00[UTC]",
            "2026-09-19T00:30:00+00:00[UTC]",
        ),
    ] {
        ctx.store.put(&event).unwrap();
    }
    let report = run_agenda(&mut ctx, args(None, None)).unwrap();
    assert_eq!(subjects(&report), ["early", "late"]);
    let later = run_agenda(&mut ctx, args(Some("2026-09-19"), None)).unwrap();
    assert_eq!(subjects(&later), ["next"]);
}

#[test]
fn the_query_narrows_the_events() {
    let mut ctx = context();
    ctx.store.put(&Object::Event(standup())).unwrap();
    ctx.store
        .put(&timed(
            5,
            "lunch",
            "2026-09-18T16:00:00+00:00[UTC]",
            "2026-09-18T17:00:00+00:00[UTC]",
        ))
        .unwrap();
    let mut narrowed = args(None, None);
    narrowed.query = Some("@work".into());
    assert_eq!(
        subjects(&run_agenda(&mut ctx, narrowed).unwrap()),
        ["standup"]
    );
}

#[test]
fn a_declined_meeting_is_listed_as_not_busy() {
    let mut ctx = context();
    let mut declined = standup();
    declined.attendees = vec![Attendee {
        email: "me@x".into(),
        response: ResponseStatus::Declined,
        is_self: true,
    }];
    ctx.store.put(&Object::Event(declined)).unwrap();
    assert_eq!(
        run_agenda(&mut ctx, args(None, None)).unwrap().data["events"][0]["busy"],
        false
    );
}

#[test]
fn a_window_that_closes_before_it_opens_is_a_usage_error() {
    let mut ctx = context();
    let err = run_agenda(&mut ctx, args(Some("2026-09-20"), Some("2026-09-19"))).unwrap_err();
    assert_eq!(err.exit_code(), 2);
    assert_eq!(err.to_string(), "--to must be later than --from");
    let past = run_agenda(&mut ctx, args(None, Some("2026-09-17"))).unwrap_err();
    assert_eq!(past.exit_code(), 2);
    assert_eq!(past.to_string(), "--to must be later than now");
    let empty = run_agenda(&mut ctx, args(Some("2026-09-20"), Some("2026-09-20"))).unwrap_err();
    assert_eq!(empty.exit_code(), 2);
}

#[test]
fn an_all_day_event_reads_as_dates() {
    let mut ctx = context();
    let mut offsite = Event::new(
        Oid::generate(&mut |b: &mut [u8]| b.fill(0x4e)),
        "offsite",
        When::Day(jiff::civil::date(2026, 9, 18)),
        When::Day(jiff::civil::date(2026, 9, 19)),
    );
    offsite.base.path = dam_domain::Path::parse("work").unwrap();
    ctx.store.put(&Object::Event(offsite)).unwrap();
    let report = run_agenda(&mut ctx, args(None, None)).unwrap();
    assert_eq!(report.data["events"][0]["all_day"], true);
    assert_eq!(
        report.human,
        "4e4e4e4  busy  2026-09-18  2026-09-19  offsite  work/"
    );
}

#[test]
fn the_local_zone_places_the_day_the_window_and_the_printed_times() {
    let mut ctx = context();
    ctx.tz = jiff::tz::TimeZone::fixed(jiff::tz::offset(-4));
    let offsite = Event::new(
        Oid::generate(&mut |b: &mut [u8]| b.fill(0x4e)),
        "offsite",
        When::Day(jiff::civil::date(2026, 9, 18)),
        When::Day(jiff::civil::date(2026, 9, 19)),
    );
    ctx.store.put(&Object::Event(offsite)).unwrap();
    ctx.store
        .put(&timed(
            2,
            "late call",
            "2026-09-18T02:00:00+00:00[UTC]",
            "2026-09-18T02:30:00+00:00[UTC]",
        ))
        .unwrap();
    ctx.store.put(&Object::Event(standup())).unwrap();
    let report = run_agenda(&mut ctx, args(Some("2026-09-18"), None)).unwrap();
    assert_eq!(subjects(&report), ["offsite", "standup"]);
    let all_day = &report.data["events"][0];
    assert_eq!(
        (all_day["start"].as_i64(), all_day["end"].as_i64()),
        (Some(1_789_704_000), Some(1_789_790_400))
    );
    assert!(
        report
            .human
            .contains("3f3f3f3  busy  2026-09-18 10:00  2026-09-18 10:30  standup"),
        "{}",
        report.human
    );
}

#[test]
fn the_human_line_says_busy_or_free_with_local_times() {
    let mut ctx = context();
    ctx.store.put(&Object::Event(standup())).unwrap();
    let report = run_agenda(&mut ctx, args(None, None)).unwrap();
    assert_eq!(
        report.human,
        "3f3f3f3  busy  2026-09-18 14:00  2026-09-18 14:30  standup"
    );
}
