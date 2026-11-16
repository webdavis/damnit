use dam_application::Window;
use jiff::SignedDuration;
use jiff::tz::TimeZone;
use serde::Serialize;

use crate::args::AgendaArgs;
use crate::commands::asked::Asked;
use crate::commands::window_edge::{Side, edge};
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{Human, View, report};
use event::AgendaEvent;

const DEFAULT_SPAN: SignedDuration = SignedDuration::from_hours(24);
const AGENDA: &str = "dam agenda";

pub(crate) fn run_agenda(ctx: &Context, args: AgendaArgs) -> Result<Report, CliError> {
    let asked = Asked {
        verb: AGENDA.into(),
        typed: &args.selection.typed,
        sort: &[],
    };
    let filter = asked.filter()?;
    let window = window(ctx, &args)?;
    let bounds: Vec<_> = args
        .max_age
        .iter()
        .map(|(remote, age)| (remote.0.clone(), *age))
        .collect();
    ctx.dam().require_fresh(&bounds)?;
    let found = ctx
        .dam()
        .agenda(&filter, &args.place, &args.project_filters, window, &ctx.tz)
        .map_err(|e| asked.refused(e))?;
    report(&Agenda {
        events: found.iter().map(|s| AgendaEvent::new(s, &ctx.tz)).collect(),
        zone: &ctx.tz,
    })
}

fn window(ctx: &Context, args: &AgendaArgs) -> Result<Window, CliError> {
    let from = match &args.from {
        Some(text) => edge(ctx, AGENDA, "from", text, Side::Opens)?,
        None => ctx.clock.now(),
    };
    let to = match &args.to {
        Some(text) => edge(ctx, AGENDA, "to", text, Side::Closes)?,
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
mod category_tests;
#[cfg(test)]
mod people_tests;
#[cfg(test)]
mod selection_tests;
#[cfg(test)]
mod tests;
