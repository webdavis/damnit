use crate::args::EditArgs;
use crate::commands::each::{report_set, run_of, selector};
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;

use super::from_flags;

pub(super) fn run_on_a_set(ctx: &Context, args: EditArgs) -> Result<Report, CliError> {
    if args.editor {
        return Err(CliError::Usage(
            "edit -e takes one oid; drop -e to change many objects with flags".into(),
        ));
    }
    let selector = selector(ctx, &args.oid, &args.set)?;
    let run = run_of(&args.set);
    let changes = ctx
        .dam()
        .edit_each(&selector, from_flags(ctx, args)?, run)?;
    report_set(ctx, &changes)
}
