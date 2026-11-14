use dam_application::Window;
use dam_domain::Timestamp;
use jiff::SignedDuration;
use jiff::tz::TimeZone;
use serde::Serialize;

use crate::args::AgendaArgs;
use crate::commands::parsing::when_flag;
use crate::commands::remote::refresh_stale_unless_no_pull;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{Human, View, report};
use event::AgendaEvent;

const DEFAULT_SPAN: SignedDuration = SignedDuration::from_hours(24);

pub(crate) fn run_agenda(ctx: &Context, args: AgendaArgs) -> Result<Report, CliError> {
    let window = window(ctx, &args)?;
    let bounds: Vec<_> = args
        .max_age
        .iter()
        .map(|(remote, age)| (remote.0.clone(), *age))
        .collect();
    refresh_stale_unless_no_pull(ctx)?;
    ctx.dam().require_fresh(&bounds)?;
    let found = ctx.dam().agenda(args.query.as_deref(), window, &ctx.tz)?;
    report(&Agenda {
        events: found.iter().map(|s| AgendaEvent::new(s, &ctx.tz)).collect(),
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

impl View for Agenda<'_> {
    fn human(&self) -> Human {
        Human::list(
            self.events.iter().map(|e| e.cells(self.zone)).collect(),
            "no events in the window",
        )
    }
}

mod event;

#[cfg(test)]
mod tests;
