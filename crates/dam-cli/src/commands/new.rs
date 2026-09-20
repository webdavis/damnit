use std::collections::BTreeSet;

use dam_application::{NewEvent, NewTask, new_event, new_task};
use dam_domain::Path;

use crate::args::NewArgs;
use crate::commands::parsing::{deadline_flag, priority_flag, when_flag};
use crate::context::Context;
use crate::error::CliError;
use crate::output::{Report, object_json};

pub(crate) fn run(ctx: &mut Context, args: NewArgs) -> Result<Report, CliError> {
    let path = Path::parse(&args.path).map_err(|e| CliError::Usage(format!("--path: {e:?}")))?;
    let labels: BTreeSet<String> = args.labels.into_iter().collect();
    let oid = if args.event {
        let start = when_flag(
            ctx,
            "start",
            args.start
                .as_deref()
                .ok_or_else(|| CliError::Usage("--event needs --start and --end".into()))?,
        )?;
        let end = when_flag(
            ctx,
            "end",
            args.end
                .as_deref()
                .ok_or_else(|| CliError::Usage("--event needs --start and --end".into()))?,
        )?;
        new_event(
            ctx.store.as_ref(),
            ctx.random.as_ref(),
            &ctx.config.categories,
            NewEvent {
                subject: args.subject,
                path,
                start,
                end,
                labels,
                body: args.body,
            },
        )?
    } else {
        let priority = args
            .priority
            .map(priority_flag)
            .transpose()?
            .unwrap_or_default();
        let due = args
            .due
            .as_deref()
            .map(|d| when_flag(ctx, "due", d))
            .transpose()?;
        let deadline = args.deadline.as_deref().map(deadline_flag).transpose()?;
        new_task(
            ctx.store.as_ref(),
            ctx.random.as_ref(),
            &ctx.config.categories,
            NewTask {
                subject: args.subject,
                path,
                priority,
                due,
                deadline,
                labels,
                body: args.body,
            },
        )?
    };
    let object = ctx
        .store
        .get(&oid)?
        .ok_or_else(|| CliError::Io("the new object did not persist".into()))?;
    Ok(Report {
        human: format!("{}  {}", oid.short(), object.base().subject),
        data: object_json(&object)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::NewArgs;
    use crate::testing::context;
    use dam_domain::Object;

    fn args(subject: &str) -> NewArgs {
        NewArgs {
            subject: subject.into(),
            event: false,
            path: String::new(),
            due: None,
            priority: None,
            deadline: None,
            labels: vec![],
            body: String::new(),
            start: None,
            end: None,
        }
    }

    #[test]
    fn a_task_lands_in_the_working_layer_with_its_flags() {
        let mut ctx = context();
        let report = run(
            &mut ctx,
            NewArgs {
                path: "inbox".into(),
                due: Some("tomorrow".into()),
                priority: Some(1),
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
        let mut ctx = context();
        assert!(matches!(
            run(
                &mut ctx,
                NewArgs {
                    event: true,
                    ..args("dentist")
                }
            ),
            Err(CliError::Usage(_))
        ));
        let report = run(
            &mut ctx,
            NewArgs {
                event: true,
                start: Some("2026-09-25T14:00".into()),
                end: Some("2026-09-25T15:00".into()),
                ..args("dentist")
            },
        )
        .unwrap();
        assert_eq!(report.data["kind"], "event");
    }

    #[test]
    fn a_bad_date_or_priority_is_a_usage_error() {
        let mut ctx = context();
        assert!(matches!(
            run(
                &mut ctx,
                NewArgs {
                    due: Some("someday".into()),
                    ..args("x")
                }
            ),
            Err(CliError::Usage(_))
        ));
        assert!(matches!(
            run(
                &mut ctx,
                NewArgs {
                    priority: Some(9),
                    ..args("x")
                }
            ),
            Err(CliError::Usage(_))
        ));
    }
}
