use dam_domain::Object;

use crate::args::LsArgs;
use crate::commands::authorship::Authorship;
use crate::commands::remote::refresh_stale_unless_no_pull;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{ObjectList, report};

pub(crate) fn run_ls(ctx: &Context, args: LsArgs) -> Result<Report, CliError> {
    refresh_stale_unless_no_pull(ctx)?;
    let authorship = Authorship::read(ctx)?;
    let objects: Vec<Object> = ctx
        .dam()
        .list(args.query.as_deref())?
        .into_iter()
        .filter(|o| authorship.by(args.by, o))
        .collect();
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

#[cfg(test)]
mod tests;
