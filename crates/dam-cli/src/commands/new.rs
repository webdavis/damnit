use std::collections::BTreeSet;

use crate::args::NewArgs;
use crate::commands::caller_id::with_caller_id;
use crate::commands::fields::fields_set;
use crate::commands::hooked::{as_it_is_now, hooked};
use crate::commands::parsing::{deadline_flag, when_flag};
use crate::commands::printed::printed_worked_out;
use crate::commands::sent::report_landed;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{Human, View, report, wire};
use dam_application::{Dam, EditFields, Landed, NewEvent, NewTask, UseCaseError};
use dam_domain::{Kind, Object};
use serde::Serialize;

#[derive(Serialize)]
#[serde(transparent)]
struct Created<'a>(#[serde(serialize_with = "wire")] &'a Object);

impl View for Created<'_> {
    fn human(&self) -> Human {
        let base = self.0.base();
        Human::Table(vec![vec![
            base.oid.short().to_string(),
            base.subject.clone(),
        ]])
    }
}

pub(crate) enum Made {
    Task(NewTask, EditFields),
    Event(NewEvent, EditFields),
}

pub(crate) fn run(ctx: &Context, args: NewArgs) -> Result<Report, CliError> {
    let id = args.id.clone();
    if args.preview {
        let hold = args.hold.clone();
        return preview::run(ctx, hold, made(ctx, args)?);
    }
    let landed = land(&ctx.dam(), made(ctx, args)?)?;
    report_landed(&landed, |object| {
        let now = as_it_is_now(ctx, object)?;
        printed_worked_out(ctx, &now, report(&hooked(ctx, Created(&now)))?)
    })
    .map(|answer| with_caller_id(answer, id.as_ref()))
}

pub(crate) fn land(dam: &Dam<'_>, made: Made) -> Result<Landed, UseCaseError> {
    match made {
        Made::Task(task, set) => dam.new_task_setting(task, set),
        Made::Event(event, set) => dam.new_event_setting(event, set),
    }
}

fn made(ctx: &Context, args: NewArgs) -> Result<Made, CliError> {
    let labels: BTreeSet<String> = args.labels.into_iter().collect();
    let parent = args
        .parent
        .as_deref()
        .map(|text| ctx.dam().resolve_object(text))
        .transpose()?;
    if args.kind == Kind::Event {
        if let Some(flag) = args.fields.task_only_flag() {
            return Err(CliError::Usage(format!(
                "{flag} is for a task; drop it, or drop --kind event"
            )));
        }
        let start = when_flag(
            ctx,
            "start",
            args.start
                .as_deref()
                .ok_or_else(|| CliError::Usage("--kind event needs --start and --end".into()))?,
        )?;
        let end = when_flag(
            ctx,
            "end",
            args.end
                .as_deref()
                .ok_or_else(|| CliError::Usage("--kind event needs --start and --end".into()))?,
        )?;
        return Ok(Made::Event(
            NewEvent {
                parent,
                subject: args.subject,
                path: args.path,
                start,
                end,
                labels,
                body: args.body,
                id: args.id,
            },
            fields_set(ctx, args.fields)?,
        ));
    }
    if args.start.is_some() || args.end.is_some() {
        return Err(CliError::Usage(
            "--start and --end need --kind event".into(),
        ));
    }
    if args.fields.location.is_some() {
        return Err(CliError::Usage("--location needs --kind event".into()));
    }
    let due = args
        .due
        .as_deref()
        .map(|d| when_flag(ctx, "due", d))
        .transpose()?;
    let deadline = args
        .deadline
        .as_deref()
        .map(|d| deadline_flag(ctx, d))
        .transpose()?;
    Ok(Made::Task(
        NewTask {
            parent,
            subject: args.subject,
            path: args.path,
            priority: args.priority.unwrap_or_default(),
            due,
            deadline,
            labels,
            body: args.body,
            id: args.id,
        },
        fields_set(ctx, args.fields)?,
    ))
}

mod preview;
pub(crate) mod required;
pub(crate) mod templated;

#[cfg(test)]
mod field_tests;
#[cfg(test)]
mod tests;
