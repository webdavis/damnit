use dam_domain::Object;

use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::stamp_worked_out;

pub(crate) fn printed_worked_out(
    ctx: &Context,
    object: &Object,
    mut printed: Report,
) -> Result<Report, CliError> {
    stamp_worked_out(&mut printed.data, &ctx.dam().worked_out_from_object(object));
    Ok(printed)
}
