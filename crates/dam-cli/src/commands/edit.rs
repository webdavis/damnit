use dam_adapters::{parse_template, render_template};
use dam_application::{EditFields, Refusal, UseCaseError, edit};
use dam_domain::Oid;

use crate::args::EditArgs;
use crate::commands::parsing::{deadline_flag, when_flag};
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
        priority: args.priority,
        ..EditFields::default()
    };
    f.due = set_or_clear(
        args.due.map(|d| when_flag(ctx, "due", &d)).transpose()?,
        args.no_due,
    );
    f.deadline = set_or_clear(
        args.deadline.map(|d| deadline_flag(ctx, &d)).transpose()?,
        args.no_deadline,
    );
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
    f.recurrence = set_or_clear(args.recurrence, args.no_recurrence);
    f.attach = set_or_clear(
        args.attach.map(|e| oid_of("attach", &e)).transpose()?,
        args.detach,
    );
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
    f.location = set_or_clear(args.location, args.no_location);
    Ok(f)
}

fn set_or_clear<T>(value: Option<T>, clear: bool) -> Option<Option<T>> {
    match (value, clear) {
        (Some(v), _) => Some(Some(v)),
        (None, true) => Some(None),
        (None, false) => None,
    }
}

fn through_editor(ctx: &Context, oid: &Oid) -> Result<EditFields, CliError> {
    let current = ctx
        .store
        .get(oid)?
        .ok_or_else(|| Refusal::NoWorkingObject(oid.short().to_string()))?;
    let editor = ctx.editor.as_ref().ok_or(Refusal::NeedsAnEditor)?;
    let original = render_template(&current);
    let parse = |saved: &str| parse_template(saved, &current, ctx.clock.today(), &ctx.tz);
    let save = |opened: &str| {
        editor
            .edit(opened)
            .map_err(|e| CliError::UseCase(UseCaseError::Editor(e)))
    };
    let saved = save(&original)?;
    if saved == original {
        return Err(CliError::Cancelled);
    }
    let why = match parse(&saved) {
        Ok(fields) => return Ok(fields),
        Err(why) => why,
    };
    let reopened = reopened_with_the_parse_error_as_its_first_line(&saved, &why);
    let saved = save(&reopened)?;
    if saved == original {
        return Err(CliError::Cancelled);
    }
    let refused = |why: String| match editor.keep(&saved) {
        Ok(path) => CliError::Usage(format!("{why}; your edit is kept at {}", path.display())),
        Err(e) => CliError::Usage(format!("{why}; your edit could not be kept: {}", e.0)),
    };
    if saved == reopened {
        return Err(refused(why));
    }
    parse(&saved).map_err(refused)
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
mod editor_tests;
#[cfg(test)]
mod tests;
