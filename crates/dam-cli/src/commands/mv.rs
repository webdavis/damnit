use crate::args::MvArgs;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{ObjectRow, report};

pub(crate) fn run(ctx: &Context, args: MvArgs) -> Result<Report, CliError> {
    if args.path.is_none() && args.parent.is_none() {
        return Err(CliError::Usage(
            "dam mv needs a path or --parent: dam mv <oid> work/clients/, or dam mv <oid> --parent <oid>".into(),
        ));
    }
    let oid = ctx.dam().resolve_object(&args.oid)?;
    let parent = args
        .parent
        .map(|text| ctx.dam().resolve_object(&text))
        .transpose()?;
    ctx.dam()
        .relocate(&oid, args.path.as_ref(), parent.as_ref())?;
    let object = ctx
        .store
        .get(&oid)?
        .ok_or_else(|| CliError::Internal("the moved object did not persist".into()))?;
    report(&ObjectRow(&object, &ctx.tz))
}

#[cfg(test)]
mod tests;
