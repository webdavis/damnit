use crate::args::MvArgs;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{ObjectRow, report};

pub(crate) fn run(ctx: &Context, args: MvArgs) -> Result<Report, CliError> {
    let oid = ctx.dam().resolve_object(&args.oid)?;
    ctx.dam().relocate(&oid, &args.path)?;
    let object = ctx
        .store
        .get(&oid)?
        .ok_or_else(|| CliError::Internal("the moved object did not persist".into()))?;
    report(&ObjectRow(&object, &ctx.tz))
}

#[cfg(test)]
mod tests;
