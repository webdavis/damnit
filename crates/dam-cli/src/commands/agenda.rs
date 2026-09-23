//! `dam agenda`: events as intervals over a window, each saying whether it
//! holds time.

use std::time::Duration;

use dam_adapters::parse_duration;
use dam_application::{Scheduled, Window, agenda, require_fresh};
use dam_domain::{Object, Timestamp};
use jiff::SignedDuration;

use crate::args::AgendaArgs;
use crate::commands::parsing::when_flag;
use crate::commands::remote::maybe_pull_stale;
use crate::context::Context;
use crate::error::CliError;
use crate::output::{Report, object_json};

/// How long the window stays open when `--to` is not given.
const DEFAULT_SPAN: SignedDuration = SignedDuration::from_hours(24);

/// The flags are read before the stale pull, so a mistyped one exits 2
/// without spawning a helper; freshness is judged after it, so a pull that
/// just landed counts.
pub(crate) fn run_agenda(ctx: &mut Context, args: AgendaArgs) -> Result<Report, CliError> {
    let window = window(ctx, &args)?;
    let bounds = max_ages(&args.max_age)?;
    maybe_pull_stale(ctx)?;
    require_fresh(ctx.store.as_ref(), ctx.clock.as_ref(), &ctx.config, &bounds)?;
    let found = agenda(
        ctx.store.as_ref(),
        ctx.clock.as_ref(),
        &ctx.config,
        args.query.as_deref(),
        window,
        &ctx.tz,
    )?;
    let rows = found.iter().map(row).collect::<Result<Vec<_>, _>>()?;
    Ok(Report {
        human: found
            .iter()
            .map(|s| line(s, &ctx.tz))
            .collect::<Vec<_>>()
            .join("\n"),
        data: serde_json::json!({ "events": rows }),
    })
}

fn window(ctx: &Context, args: &AgendaArgs) -> Result<Window, CliError> {
    let instant = |flag: &str, text: &str| -> Result<Timestamp, CliError> {
        when_flag(ctx, flag, text)?
            .instant(&ctx.tz)
            .ok_or_else(|| CliError::Usage(format!("--{flag}: {text} is outside the calendar")))
    };
    let from = match &args.from {
        Some(text) => instant("from", text)?,
        None => ctx.clock.now(),
    };
    let to = match &args.to {
        Some(text) => instant("to", text)?,
        None => from.saturating_add(DEFAULT_SPAN).unwrap_or(from),
    };
    if to <= from {
        return Err(CliError::Usage("--to must be later than --from".into()));
    }
    Ok(Window { from, to })
}

/// Each `--max-age <remote>=<duration>`, the duration read the way `stale` is.
fn max_ages(texts: &[String]) -> Result<Vec<(String, Duration)>, CliError> {
    texts
        .iter()
        .map(|text| {
            let unreadable = || {
                CliError::Usage(format!(
                    "--max-age {text:?}: expected <remote>=<number><s|m|h>"
                ))
            };
            let (remote, age) = text.split_once('=').ok_or_else(unreadable)?;
            if remote.is_empty() {
                return Err(unreadable());
            }
            let age = parse_duration("--max-age", age).map_err(|_| unreadable())?;
            Ok((remote.to_string(), age))
        })
        .collect()
}

/// Dam's own summary of the event beside the three keys a scheduler reads.
fn row(s: &Scheduled) -> Result<serde_json::Value, CliError> {
    let wire = object_json(&Object::Event(s.event.clone()))?;
    Ok(serde_json::json!({
        "oid": wire["oid"],
        "subject": wire["subject"],
        "path": wire["path"],
        "labels": wire["labels"],
        "status": wire["event"]["status"],
        "transparency": wire["event"]["transparency"],
        "all_day": s.event.start.is_all_day(),
        "start": s.start.as_second(),
        "end": s.end.as_second(),
        "busy": s.busy,
    }))
}

fn line(s: &Scheduled, zone: &jiff::tz::TimeZone) -> String {
    let local = |t: Timestamp| {
        let z = t.to_zoned(zone.clone());
        if s.event.start.is_all_day() {
            z.strftime("%Y-%m-%d").to_string()
        } else {
            z.strftime("%Y-%m-%d %H:%M").to_string()
        }
    };
    let mut cols = vec![
        s.event.base.oid.short().to_string(),
        if s.busy { "busy" } else { "free" }.to_string(),
        local(s.start),
        local(s.end),
        s.event.base.subject.clone(),
    ];
    if !s.event.base.path.as_str().is_empty() {
        cols.push(s.event.base.path.as_str().to_string());
    }
    cols.join("  ")
}

#[cfg(test)]
mod tests {
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

    /// The contract, whole: a change to any key here is a change to what
    /// programs reading busy times rely on.
    #[test]
    fn the_agenda_document_keeps_its_contract() {
        let mut ctx = context(); // now is 2026-09-18T00:00:00Z
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

    /// Now is midnight, so the window runs to the next midnight and no
    /// further.
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
        assert!(err.to_string().contains("--to"), "{err}");
        // A window that closes as it opens holds no instant either.
        let empty = run_agenda(&mut ctx, args(Some("2026-09-20"), Some("2026-09-20"))).unwrap_err();
        assert_eq!(empty.exit_code(), 2);
    }

    /// An all-day row says so, and its human line shows dates and the path.
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
    fn the_human_line_says_busy_or_free_with_local_times() {
        let mut ctx = context();
        ctx.store.put(&Object::Event(standup())).unwrap();
        let report = run_agenda(&mut ctx, args(None, None)).unwrap();
        assert_eq!(
            report.human,
            "3f3f3f3  busy  2026-09-18 14:00  2026-09-18 14:30  standup"
        );
    }
}
