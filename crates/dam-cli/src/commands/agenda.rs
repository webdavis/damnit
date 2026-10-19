use std::collections::BTreeSet;

use dam_application::{Scheduled, Window, agenda, require_fresh};
use dam_domain::{Oid, Timestamp};
use jiff::SignedDuration;
use jiff::tz::TimeZone;
use serde::Serialize;

use crate::args::AgendaArgs;
use crate::commands::parsing::when_flag;
use crate::commands::remote::refresh_stale_unless_no_pull;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{Human, View, display, report};

const DEFAULT_SPAN: SignedDuration = SignedDuration::from_hours(24);

pub(crate) fn run_agenda(ctx: &Context, args: AgendaArgs) -> Result<Report, CliError> {
    let window = window(ctx, &args)?;
    let bounds: Vec<_> = args
        .max_age
        .iter()
        .map(|(remote, age)| (remote.0.clone(), *age))
        .collect();
    refresh_stale_unless_no_pull(ctx)?;
    require_fresh(ctx.store.as_ref(), ctx.clock.as_ref(), &ctx.config, &bounds)?;
    let found = agenda(
        ctx.store.as_ref(),
        ctx.clock.as_ref(),
        &ctx.config,
        args.query.as_deref(),
        window,
        &ctx.tz,
    )?;
    report(&Agenda {
        events: found.iter().map(AgendaEvent::new).collect(),
        zone: &ctx.tz,
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
        let opening = if args.from.is_some() { "--from" } else { "now" };
        return Err(CliError::Usage(format!(
            "--to must be later than {opening}"
        )));
    }
    Ok(Window { from, to })
}

#[derive(Serialize)]
struct Agenda<'a> {
    events: Vec<AgendaEvent<'a>>,
    #[serde(skip)]
    zone: &'a TimeZone,
}

#[derive(Serialize)]
struct AgendaEvent<'a> {
    #[serde(skip)]
    scheduled: &'a Scheduled,
    #[serde(serialize_with = "display")]
    oid: &'a Oid,
    subject: &'a str,
    path: &'a str,
    labels: &'a BTreeSet<String>,
    status: &'static str,
    transparency: &'static str,
    all_day: bool,
    start: i64,
    end: i64,
    busy: bool,
}

impl<'a> AgendaEvent<'a> {
    fn new(s: &'a Scheduled) -> AgendaEvent<'a> {
        let base = &s.event.base;
        AgendaEvent {
            scheduled: s,
            oid: &base.oid,
            subject: &base.subject,
            path: base.path.as_str(),
            labels: &base.labels,
            status: s.event.status.as_str(),
            transparency: s.event.transparency.as_str(),
            all_day: s.event.start.is_all_day(),
            start: s.start.as_second(),
            end: s.end.as_second(),
            busy: s.busy,
        }
    }

    fn cells(&self, zone: &TimeZone) -> Vec<String> {
        let s = self.scheduled;
        let local = |t: Timestamp| {
            let z = t.to_zoned(zone.clone());
            if self.all_day {
                z.strftime("%Y-%m-%d").to_string()
            } else {
                z.strftime("%Y-%m-%d %H:%M").to_string()
            }
        };
        vec![
            self.oid.short().to_string(),
            if self.busy { "busy" } else { "free" }.to_string(),
            local(s.start),
            local(s.end),
            self.path.to_string(),
            self.subject.to_string(),
        ]
    }
}

impl View for Agenda<'_> {
    fn human(&self) -> Human {
        Human::list(
            self.events.iter().map(|e| e.cells(self.zone)).collect(),
            "no events in the window",
        )
    }
}

#[cfg(test)]
mod tests;
