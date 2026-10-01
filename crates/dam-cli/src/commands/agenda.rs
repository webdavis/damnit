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

const DEFAULT_SPAN: SignedDuration = SignedDuration::from_hours(24);

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
        let opening = if args.from.is_some() { "--from" } else { "now" };
        return Err(CliError::Usage(format!(
            "--to must be later than {opening}"
        )));
    }
    Ok(Window { from, to })
}

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
mod tests;
