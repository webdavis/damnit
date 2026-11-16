use dam_application::{Agenda as Found, Window};
use dam_domain::Scope;
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
use task::AgendaTask;

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
        events: found
            .events
            .iter()
            .map(|s| AgendaEvent::new(s, &ctx.tz))
            .collect(),
        tasks: found.scope.tasks.then(|| {
            found
                .tasks
                .iter()
                .map(|d| AgendaTask::new(d, &ctx.tz))
                .collect()
        }),
        found: &found,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    tasks: Option<Vec<AgendaTask>>,
    #[serde(skip)]
    found: &'a Found,
    #[serde(skip)]
    zone: &'a TimeZone,
}

impl View for Agenda<'_> {
    fn human(&self) -> Human {
        let mut rows = Vec::new();
        let printed = self.tasks.iter().flatten();
        let mut tasks = self.found.tasks.iter().zip(printed).peekable();
        for (scheduled, event) in self.found.events.iter().zip(&self.events) {
            while let Some((_, task)) = tasks.next_if(|(due, _)| due.day < scheduled.day) {
                rows.push(task.cells());
            }
            rows.push(event.cells(self.zone));
        }
        rows.extend(tasks.map(|(_, task)| task.cells()));
        Human::list(rows, nothing(self.found.scope))
    }
}

fn nothing(scope: Scope) -> &'static str {
    match (scope.tasks, scope.events) {
        (false, _) => "no events in the window",
        (true, false) => "no tasks due in the window",
        (true, true) => "no events or tasks due in the window",
    }
}

mod event;
mod task;

#[cfg(test)]
mod category_tests;
#[cfg(test)]
mod people_tests;
#[cfg(test)]
mod selection_tests;
#[cfg(test)]
mod tasks_tests;
#[cfg(test)]
mod tests;
