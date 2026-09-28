use dam_adapters::{parse_template, render_template};
use dam_application::{EditFields, Refusal, UseCaseError, edit};
use dam_domain::Oid;

use crate::args::EditArgs;
use crate::commands::parsing::{deadline_flag, priority_flag, when_flag};
use crate::context::Context;
use crate::error::CliError;
use crate::oids::resolve_oid;
use crate::output::{Report, object_json, object_line};

pub(crate) fn run(ctx: &mut Context, args: EditArgs) -> Result<Report, CliError> {
    let oid = resolve_oid(ctx.store.as_ref(), &args.oid)?;
    let fields = if args.editor {
        through_editor(ctx, &oid)?
    } else {
        from_flags(ctx, args)?
    };
    let object = edit(ctx.store.as_ref(), &ctx.config.categories, &oid, fields)?;
    Ok(Report {
        human: object_line(&object),
        data: object_json(&object)?,
    })
}

fn from_flags(ctx: &Context, args: EditArgs) -> Result<EditFields, CliError> {
    let oid_of = |flag: &str, text: &str| {
        resolve_oid(ctx.store.as_ref(), text).map_err(|e| CliError::Usage(format!("--{flag}: {e}")))
    };
    let mut f = EditFields {
        subject: args.subject,
        body: args.body,
        ..EditFields::default()
    };
    f.priority = args.priority.map(priority_flag).transpose()?;
    f.due = match (args.due, args.no_due) {
        (Some(d), _) => Some(Some(when_flag(ctx, "due", &d)?)),
        (None, true) => Some(None),
        (None, false) => None,
    };
    f.deadline = match (args.deadline, args.no_deadline) {
        (Some(d), _) => Some(Some(deadline_flag(ctx, &d)?)),
        (None, true) => Some(None),
        (None, false) => None,
    };
    f.undone = args.undone;
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
        .map(|s| when_flag(ctx, "start", s))
        .transpose()?;
    f.end = args
        .end
        .as_deref()
        .map(|s| when_flag(ctx, "end", s))
        .transpose()?;
    f.location = match (args.location, args.no_location) {
        (Some(l), _) => Some(Some(l)),
        (None, true) => Some(None),
        (None, false) => None,
    };
    Ok(f)
}

fn through_editor(ctx: &Context, oid: &Oid) -> Result<EditFields, CliError> {
    let current = ctx
        .store
        .get(oid)?
        .ok_or_else(|| Refusal::NoWorkingObject(oid.short().to_string()))?;
    let editor = ctx.editor.as_ref().ok_or(Refusal::NeedsAnEditor)?;
    let original = render_template(&current);
    let mut opened = original.clone();
    loop {
        let saved = editor
            .edit(&opened)
            .map_err(|e| CliError::UseCase(UseCaseError::Editor(e)))?;
        if saved == opened || saved == original {
            return Err(CliError::Cancelled);
        }
        match parse_template(&saved, &current, ctx.clock.today(), &ctx.tz) {
            Ok(fields) => return Ok(fields),
            Err(why) => {
                let reopened = reopened_with_the_parse_error_as_its_first_line(&saved, &why);
                if reopened == opened {
                    return Err(CliError::Usage(why));
                }
                opened = reopened;
            }
        }
    }
}

fn reopened_with_the_parse_error_as_its_first_line(saved: &str, why: &str) -> String {
    let body = saved
        .lines()
        .filter(|l| !l.starts_with("# error:"))
        .collect::<Vec<_>>()
        .join("\n");
    format!("# error: {why}\n{body}\n")
}

#[cfg(test)]
mod tests;
