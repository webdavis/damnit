use crate::args::MvArgs;
use crate::args::values::path_arg;
use crate::commands::each::{one_object, report_set, run_of, selector};
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{ObjectRow, report};

pub(crate) fn run(ctx: &Context, mut args: MvArgs) -> Result<Report, CliError> {
    let path = match (args.path.take(), &args.parent) {
        (Some(path), _) => Some(path),
        (None, Some(_)) => None,
        (None, None) if args.oid.len() >= 2 => {
            let last = args.oid.pop().expect("two or more words");
            Some(path_arg(&last).map_err(|why| CliError::Usage(format!("{last}: {why}")))?)
        }
        (None, None) => {
            return Err(CliError::Usage(
                "dam mv needs a path or --parent: dam mv <oid> work/clients/, or dam mv <oid> --parent <oid>".into(),
            ));
        }
    };
    let parent = args
        .parent
        .as_ref()
        .map(|text| ctx.dam().resolve_object(text))
        .transpose()?;
    let Some(oid) = one_object(&args.oid, &args.set) else {
        let selector = selector(ctx, &args.oid, &args.set)?;
        let changes = ctx.dam().relocate_each(
            &selector,
            path.as_ref(),
            parent.as_ref(),
            run_of(&args.set),
        )?;
        return report_set(ctx, &changes);
    };
    let oid = ctx.dam().resolve_object(oid)?;
    ctx.dam().relocate(&oid, path.as_ref(), parent.as_ref())?;
    let object = ctx
        .store
        .get(&oid)?
        .ok_or_else(|| CliError::Internal("the moved object did not persist".into()))?;
    report(&ObjectRow(&object, &ctx.tz))
}

#[cfg(test)]
mod tests;
