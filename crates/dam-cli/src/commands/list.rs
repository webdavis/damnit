use dam_application::{SelectError, UseCaseError};
use dam_domain::{Filter, Object, Order};

use crate::args::ListArgs;
use crate::commands::authorship::Authorship;
use crate::context::Context;
use crate::error::{CliError, Fault};
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
        return Ok(ctx.dam().list_within(Some(query), &args.place)?);
    }
    let typed = &args.selection.typed;
    let fault = |error| {
        CliError::Selection(Box::new(Fault {
            error,
            typed: typed.clone(),
        }))
    };
    let filter = Filter::read(typed).map_err(|e| fault(SelectError::Value(e)))?;
    let dam = ctx.dam();
    let listed = dam.list_selected(
        &filter,
        &Order::default(),
        &args.place,
        &args.project_filters,
    );
    listed.map_err(|e| match e {
        UseCaseError::Select(error) => fault(error),
        e => e.into(),
    })
}

#[cfg(test)]
mod tests;
