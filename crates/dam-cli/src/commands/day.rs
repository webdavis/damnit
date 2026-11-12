use dam_adapters::{DayDoc, day_from_doc, day_to_doc};
use dam_application::UseCaseError;
use dam_domain::Day;
use serde::Serialize;

use crate::args::{DayArgs, DayCommand};
use crate::commands::parsing::day_flag;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::request;
use crate::view::{Human, View, report};

#[derive(Serialize)]
#[serde(transparent)]
struct DayView(DayDoc);

impl View for DayView {
    fn human(&self) -> Human {
        let d = &self.0;
        let mut lines = vec![
            format!("day: {}", d.day),
            format!("closed_at: {}", d.closed_at.as_deref().unwrap_or("open")),
            format!("plan: {}", d.plan.join(", ")),
            format!("proposals: {}", d.proposals.join(", ")),
        ];
        lines.extend(d.decisions.iter().map(|text| format!("decision: {text}")));
        lines.extend(d.tasks.iter().map(|t| {
            format!(
                "task {}: stalls {}, carried_forward {}",
                t.oid, t.stalls, t.carried_forward
            )
        }));
        Human::Lines(lines)
    }
}

#[derive(Serialize)]
struct DayList {
    days: Vec<Closed>,
}

#[derive(Serialize)]
struct Closed {
    day: String,
    closed_at: Option<String>,
}

impl View for DayList {
    fn human(&self) -> Human {
        let rows = self
            .days
            .iter()
            .map(|d| {
                vec![
                    d.day.clone(),
                    d.closed_at.clone().unwrap_or_else(|| "open".into()),
                ]
            })
            .collect();
        Human::list(rows, "no day is recorded")
    }
}

pub(crate) fn run_day(ctx: &Context, args: DayArgs) -> Result<Report, CliError> {
    match args.command {
        DayCommand::Show { day } => {
            let on = match day {
                Some(text) => day_flag(ctx, "day", &text)?,
                None => ctx.clock.today(),
            };
            shown(ctx.dam().day(on)?)
        }
        DayCommand::Set { document } => {
            let text = match document {
                Some(text) => text,
                None => request::read_stdin()?,
            };
            shown(ctx.dam().record_day(read_document(&text)?)?)
        }
        DayCommand::List => {
            let days = ctx.dam().days_newest_first()?;
            report(&DayList {
                days: days
                    .iter()
                    .map(|d| Closed {
                        day: d.day.to_string(),
                        closed_at: d.closed_at.map(|at| at.to_string()),
                    })
                    .collect(),
            })
        }
    }
}

fn shown(day: Day) -> Result<Report, CliError> {
    report(&DayView(day_to_doc(&day)))
}

pub(crate) fn read_document(text: &str) -> Result<Day, CliError> {
    request::decode::<DayDoc>(text)
        .and_then(day_from_doc)
        .map_err(|why| CliError::UseCase(UseCaseError::Parse(format!("day document: {why}"))))
}
