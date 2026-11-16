use dam_application::SelectError;
use dam_domain::{Object, Order, Sort};

use crate::args::ListArgs;
use crate::commands::asked::Asked;
use crate::commands::authorship::Authorship;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{ObjectList, report};

pub(crate) fn run_list(ctx: &Context, args: ListArgs) -> Result<Report, CliError> {
    let objects = listed(ctx, &args)?;
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
    }
    Ok(listed)
}

fn listed(ctx: &Context, args: &ListArgs) -> Result<Vec<Object>, CliError> {
    if let Some(query) = &args.project_default {
        if let Some(refused) = order_beside_default::refused(&args.order) {
            return Err(refused);
        }
        return Ok(ctx.dam().list_within(Some(query), &args.place)?);
    }
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
        .list_selected(&filter, &order, &args.place, &args.project_filters)
        .map_err(|e| asked.refused(e))
}

mod order_beside_default;

#[cfg(test)]
mod order_tests;
#[cfg(test)]
mod tests;
