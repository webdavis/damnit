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
mod tests {
    use super::*;
    use crate::args::NewArgs;
    use crate::testing::context;
    use dam_domain::{Path, Priority};

    fn args(subject: &str) -> NewArgs {
        NewArgs {
            subject: subject.into(),
            kind: Kind::Task,
            event: false,
            path: Path::root(),
            due: None,
            priority: None,
            deadline: None,
            labels: vec![],
            body: String::new(),
            start: None,
            end: None,
            id: None,
        }
    }

    #[test]
    fn a_task_lands_in_the_working_layer_with_its_flags() {
        let ctx = context();
        let report = run(
            &ctx,
            NewArgs {
                path: Path::parse("inbox").unwrap(),
                due: Some("tomorrow".into()),
                priority: Some(Priority::HIGHEST),
                labels: vec!["errand".into()],
                ..args("milk")
            },
        )
        .unwrap();
        let all = ctx.store.all().unwrap();
        assert_eq!(all.len(), 1);
        let Object::Task(t) = &all[0] else {
            panic!("not a task")
        };
        assert_eq!(t.base.subject, "milk");
        assert_eq!(t.base.path.as_str(), "inbox/");
        assert_eq!(t.priority.get(), 1);
        assert_eq!(
            t.due.as_ref().map(|d| d.to_text()).as_deref(),
            Some("2026-09-19")
        );
        assert!(t.base.labels.contains("errand"));
        assert!(report.human.starts_with(t.base.oid.short()));
        assert_eq!(report.data["subject"], "milk");
    }

    #[test]
    fn an_event_needs_start_and_end() {
        let ctx = context();
        assert!(matches!(
            run(
                &ctx,
                NewArgs {
                    kind: Kind::Event,
                    ..args("dentist")
                }
            ),
            Err(CliError::Usage(_))
        ));
        let report = run(
            &ctx,
            NewArgs {
                kind: Kind::Event,
                start: Some("2026-09-25T14:00".into()),
                end: Some("2026-09-25T15:00".into()),
                ..args("dentist")
            },
        )
        .unwrap();
        assert_eq!(report.data["kind"], "event");
    }

    #[test]
    fn a_task_given_a_start_is_a_usage_error() {
        let ctx = context();
        let started = NewArgs {
            start: Some("2026-09-25T14:00".into()),
            ..args("dentist")
        };
        assert!(matches!(run(&ctx, started), Err(CliError::Usage(_))));
    }

    #[test]
    fn a_bad_date_is_a_usage_error() {
        let ctx = context();
        assert!(matches!(
            run(
                &ctx,
                NewArgs {
                    due: Some("someday".into()),
                    ..args("x")
                }
            ),
            Err(CliError::Usage(_))
        ));
    }
}
