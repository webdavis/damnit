use dam_application::{BeyondPulled, Listed, SelectError};
use dam_domain::{Order, Sort};

use crate::args::ListArgs;
use crate::commands::asked::Asked;
use crate::commands::authorship::Authorship;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{ObjectList, report, stamp_worked_out};

pub(crate) fn run_list(ctx: &Context, args: ListArgs) -> Result<Report, CliError> {
    let (
        Listed {
            objects,
            worked_out,
        },
        beyond,
    ) = listed(ctx, &args)?;
    let authorship = Authorship::read(ctx)?;
    let authors: Vec<&str> = objects.iter().map(|o| authorship.author(o.oid())).collect();
    let mut listed = report(&ObjectList {
        objects: &objects,
        authors: &authors,
        zone: &ctx.tz,
    })?;
    if let Some(rows) = listed.data["objects"].as_array_mut() {
        for (row, object) in rows.iter_mut().zip(&objects) {
            authorship.stamp(object.oid(), row)?;
        }
        for (row, worked) in rows.iter_mut().zip(&worked_out) {
            stamp_worked_out(row, worked);
        }
    }
    let notes = beyond.iter().map(beyond_pulled::warning);
    listed.standard_error_lines = notes.chain(listed.standard_error_lines).collect();
    Ok(listed)
}

fn listed(ctx: &Context, args: &ListArgs) -> Result<(Listed, Option<BeyondPulled>), CliError> {
    let asked = Asked {
        verb: "dam list".into(),
        typed: &args.selection.typed,
        sort: &args.order.sort,
    };
    let filter = asked.filter()?;
    let order = Order {
        sort: Sort::read(&args.order.sort).map_err(|e| asked.fault(SelectError::Sort(e)))?,
        offset: args.order.offset.unwrap_or(0),
        limit: args.order.limit,
    };
    ctx.dam()
        .list_selected_noting(&filter, &order, &args.place, &args.project_filters)
        .map_err(|e| asked.refused(e))
}

mod beyond_pulled;
#[cfg(test)]
mod days_and_children_tests;

#[cfg(test)]
mod order_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod worked_out_tests;
