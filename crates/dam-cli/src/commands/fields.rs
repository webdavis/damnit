use dam_application::EditFields;
use dam_domain::Oid;

use crate::args::FieldArgs;
use crate::commands::parsing::day_flag;
use crate::context::Context;
use crate::error::{CliError, UseCaseText};

pub(crate) fn fields_set(ctx: &Context, args: FieldArgs) -> Result<EditFields, CliError> {
    Ok(EditFields {
        add_depends: args
            .depends
            .iter()
            .map(|d| oid_of(ctx, "depends", d))
            .collect::<Result<Vec<Oid>, _>>()?,
        recurrence: args.recurrence.map(Some),
        attach: args
            .attached
            .map(|e| oid_of(ctx, "attached", &e).map(Some))
            .transpose()?,
        location: args.location.map(Some),
        estimate: args.estimate.map(Some),
        focus: args
            .focus
            .map(|d| day_flag(ctx, "focus", &d).map(Some))
            .transpose()?,
        ..EditFields::default()
    })
}

pub(crate) fn oid_of(ctx: &Context, flag: &str, text: &str) -> Result<Oid, CliError> {
    ctx.dam()
        .resolve_object(text)
        .map_err(|e| CliError::Usage(format!("--{flag}: {}", UseCaseText(&e))))
}
