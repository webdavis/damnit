use dam_adapters::{parse_template, render_template};
use dam_application::{EditFields, edit};
use dam_domain::{Date, Oid, Priority, When};

use crate::args::EditArgs;
use crate::context::Context;
use crate::error::CliError;
use crate::oids::resolve_oid;
use crate::output::{Report, object_json, object_line};

pub fn run(ctx: &mut Context, args: EditArgs) -> Result<Report, CliError> {
    let oid = resolve_oid(ctx.store.as_ref(), &args.oid)?;
    let fields = if args.editor {
        through_editor(ctx, &oid)?
    } else {
        from_flags(ctx, args)?
    };
    let object = edit(ctx.store.as_ref(), &ctx.config.categories, &oid, fields)?;
    Ok(Report {
        human: object_line(&object),
        data: object_json(&object),
    })
}

fn from_flags(ctx: &Context, args: EditArgs) -> Result<EditFields, CliError> {
    let today = ctx.clock.today();
    let when = |flag: &str, text: &str| {
        When::parse_human(text, today, &ctx.tz)
            .map_err(|e| CliError::Usage(format!("--{flag}: {}", e.0)))
    };
    let oid_of = |flag: &str, text: &str| {
        resolve_oid(ctx.store.as_ref(), text).map_err(|e| CliError::Usage(format!("--{flag}: {e}")))
    };
    let mut f = EditFields {
        subject: args.subject,
        body: args.body,
        ..EditFields::default()
    };
    f.priority = args
        .priority
        .map(|p| {
            Priority::new(p)
                .map_err(|_| CliError::Usage(format!("-p {p}: priority is 1 (highest) to 4")))
        })
        .transpose()?;
    f.due = match (args.due, args.no_due) {
        (Some(d), _) => Some(Some(when("due", &d)?)),
        (None, true) => Some(None),
        (None, false) => None,
    };
    f.deadline = match (args.deadline, args.no_deadline) {
        (Some(d), _) => Some(Some(
            d.parse::<Date>()
                .map_err(|e| CliError::Usage(format!("--deadline: {e}")))?,
        )),
        (None, true) => Some(None),
        (None, false) => None,
    };
    f.add_labels = args.labels;
    f.remove_labels = args.unlabels;
    f.add_depends = args
        .depends
        .iter()
        .map(|d| oid_of("depends", d))
        .collect::<Result<Vec<Oid>, _>>()?;
    f.remove_depends = args
        .undepends
        .iter()
        .map(|d| oid_of("undepends", d))
        .collect::<Result<Vec<Oid>, _>>()?;
    f.recurrence = match (args.recurrence, args.no_recurrence) {
        (Some(r), _) => Some(Some(r)),
        (None, true) => Some(None),
        (None, false) => None,
    };
    f.attach = match (args.attach, args.detach) {
        (Some(e), _) => Some(Some(oid_of("attach", &e)?)),
        (None, true) => Some(None),
        (None, false) => None,
    };
    f.start = args
        .start
        .as_deref()
        .map(|s| when("start", s))
        .transpose()?;
    f.end = args.end.as_deref().map(|s| when("end", s)).transpose()?;
    f.location = match (args.location, args.no_location) {
        (Some(l), _) => Some(Some(l)),
        (None, true) => Some(None),
        (None, false) => None,
    };
    Ok(f)
}

/// Reopens on a parse failure with the error on the first comment line; a save
/// identical to what was opened is a cancel.
fn through_editor(ctx: &Context, oid: &Oid) -> Result<EditFields, CliError> {
    let current = ctx
        .store
        .get(oid)?
        .ok_or_else(|| CliError::Usage(format!("{} is not in the working layer", oid.short())))?;
    let original = render_template(&current);
    let mut opened = original.clone();
    loop {
        let saved = ctx.editor.edit(&opened).map_err(|e| CliError::Io(e.0))?;
        if saved == opened || saved == original {
            return Err(CliError::Cancelled);
        }
        match parse_template(&saved, &current, ctx.clock.today(), &ctx.tz) {
            Ok(fields) => return Ok(fields),
            Err(why) => {
                let body = saved
                    .lines()
                    .filter(|l| !l.starts_with("# error:"))
                    .collect::<Vec<_>>()
                    .join("\n");
                let reopened = format!("# error: {why}\n{body}\n");
                if reopened == opened {
                    return Err(CliError::Usage(why));
                }
                opened = reopened;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::EditArgs;
    use crate::testing::{ScriptedEditor, context};
    use dam_domain::{Object, Oid, Task};

    fn oid(b: u8) -> Oid {
        Oid::generate(&mut |x: &mut [u8]| x.fill(b))
    }

    fn args(oid: &Oid) -> EditArgs {
        EditArgs {
            oid: oid.as_str().into(),
            editor: false,
            subject: None,
            body: None,
            priority: None,
            due: None,
            no_due: false,
            deadline: None,
            no_deadline: false,
            labels: vec![],
            unlabels: vec![],
            depends: vec![],
            undepends: vec![],
            recurrence: None,
            no_recurrence: false,
            attach: None,
            detach: false,
            start: None,
            end: None,
            location: None,
            no_location: false,
        }
    }

    #[test]
    fn flags_change_fields() {
        let mut ctx = context();
        ctx.store
            .put(&Object::Task(Task::new(oid(1), "milk")))
            .unwrap();
        run(
            &mut ctx,
            EditArgs {
                subject: Some("oat milk".into()),
                due: Some("2026-09-25".into()),
                labels: vec!["errand".into()],
                ..args(&oid(1))
            },
        )
        .unwrap();
        let t = ctx.store.get(&oid(1)).unwrap().unwrap();
        assert_eq!(t.base().subject, "oat milk");
        assert!(t.base().labels.contains("errand"));
        assert_eq!(
            t.as_task()
                .unwrap()
                .due
                .as_ref()
                .map(|d| d.to_text())
                .as_deref(),
            Some("2026-09-25")
        );
    }

    #[test]
    fn the_editor_round_trip_applies_the_saved_text() {
        let mut ctx = context();
        ctx.store
            .put(&Object::Task(Task::new(oid(1), "milk")))
            .unwrap();
        let saved = dam_adapters::render_template(&ctx.store.get(&oid(1)).unwrap().unwrap())
            .replace("subject = \"milk\"", "subject = \"eggs\"");
        ctx.editor = Box::new(ScriptedEditor(saved));
        run(
            &mut ctx,
            EditArgs {
                editor: true,
                ..args(&oid(1))
            },
        )
        .unwrap();
        assert_eq!(
            ctx.store.get(&oid(1)).unwrap().unwrap().base().subject,
            "eggs"
        );
    }

    #[test]
    fn an_unchanged_editor_save_is_a_cancel() {
        let mut ctx = context();
        ctx.store
            .put(&Object::Task(Task::new(oid(1), "milk")))
            .unwrap();
        let same = dam_adapters::render_template(&ctx.store.get(&oid(1)).unwrap().unwrap());
        ctx.editor = Box::new(ScriptedEditor(same));
        assert!(matches!(
            run(
                &mut ctx,
                EditArgs {
                    editor: true,
                    ..args(&oid(1))
                }
            ),
            Err(CliError::Cancelled)
        ));
    }

    #[test]
    fn a_save_that_never_parses_stops_after_it_comes_back_unchanged() {
        let mut ctx = context();
        ctx.store
            .put(&Object::Task(Task::new(oid(1), "milk")))
            .unwrap();
        ctx.editor = Box::new(ScriptedEditor("priority = \"high\"\n".into()));
        let err = run(
            &mut ctx,
            EditArgs {
                editor: true,
                ..args(&oid(1))
            },
        )
        .unwrap_err();
        assert!(err.to_string().contains("priority"), "{err}");
        assert_eq!(
            ctx.store.get(&oid(1)).unwrap().unwrap().base().subject,
            "milk"
        );
    }
}
