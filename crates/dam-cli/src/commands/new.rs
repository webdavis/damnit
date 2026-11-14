use std::collections::BTreeSet;

use crate::args::NewArgs;
use crate::commands::caller_id::with_caller_id;
use crate::commands::parsing::{deadline_flag, when_flag};
use crate::commands::sent::report_landed;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{Human, View, report, wire};
use dam_application::{NewEvent, NewTask};
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

pub(crate) fn run(ctx: &Context, args: NewArgs) -> Result<Report, CliError> {
    let labels: BTreeSet<String> = args.labels.into_iter().collect();
    let id = args.id.clone();
    let landed = if args.kind == Kind::Event {
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
        ctx.dam().new_event(NewEvent {
            subject: args.subject,
            path: args.path,
            start,
            end,
            labels,
            body: args.body,
            id: args.id,
        })?
    } else {
        if args.start.is_some() || args.end.is_some() {
            return Err(CliError::Usage(
                "--start and --end need --kind event".into(),
            ));
        }
        let priority = args.priority.unwrap_or_default();
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
        ctx.dam().new_task(NewTask {
            subject: args.subject,
            path: args.path,
            priority,
            due,
            deadline,
            labels,
            body: args.body,
            id: args.id,
        })?
    };
    report_landed(&landed, |object| report(&Created(object)))
        .map(|answer| with_caller_id(answer, id.as_ref()))
}

#[cfg(test)]
mod tests;
