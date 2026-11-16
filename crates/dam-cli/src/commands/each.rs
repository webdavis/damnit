use dam_application::{Selector, SetChanges, SetRun};

use crate::args::SetArgs;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::report;
use view::SetView;

pub(crate) fn one_object<'a>(oids: &'a [String], set: &SetArgs) -> Option<&'a str> {
    match oids {
        [oid] if set.is_plain() => Some(oid),
        _ => None,
    }
}

pub(crate) fn selector(
    ctx: &Context,
    oids: &[String],
    set: &SetArgs,
) -> Result<Selector, CliError> {
    Ok(match (&set.query, &set.filter) {
        (Some(query), _) => Selector::Query(query.clone()),
        (_, Some(filter)) => Selector::Filter(filter.clone()),
        _ if oids.is_empty() => {
            return Err(CliError::Usage(
                "name the objects: give oids, --query TEXT or --filter NAME".into(),
            ));
        }
        _ => Selector::Oids(
            oids.iter()
                .map(|text| ctx.dam().resolve_object(text))
                .collect::<Result<_, _>>()?,
        ),
    })
}

pub(crate) fn run_of(set: &SetArgs) -> SetRun {
    SetRun {
        preview: set.preview,
        expect: set.expect,
    }
}

pub(crate) fn report_set(ctx: &Context, changes: &SetChanges) -> Result<Report, CliError> {
    report(&SetView::of(changes, ctx.config.owner_name()))
}

mod view;

#[cfg(test)]
mod tests;
