use dam_application::{Selector, SetChange, SetChanges, SetRun};
use dam_domain::Change;

use crate::args::SetArgs;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::report;
use confirm::confirmed;
pub(crate) use view::SetView;

pub(crate) fn one_object<'a>(oids: &'a [String], set: &SetArgs) -> Option<&'a str> {
    match oids {
        [oid] if set.is_plain() => Some(oid),
        _ => None,
    }
}

pub(crate) fn set_of(
    ctx: &Context,
    oids: &[String],
    set: &SetArgs,
) -> Result<(Selector, SetRun), CliError> {
    if set.as_one && ctx.access.token().is_none() {
        return Err(CliError::Usage(
            "--as-one is for a token; your own changes do not wait, so drop --as-one".into(),
        ));
    }
    let selector = selector(ctx, oids, set)?;
    let run = SetRun {
        preview: set.preview,
        expect: set.expect,
        as_one: set.as_one,
    };
    let run = confirmed(ctx, &selector, run, set.piped.as_deref())?;
    Ok((selector, run))
}

fn selector(ctx: &Context, oids: &[String], set: &SetArgs) -> Result<Selector, CliError> {
    let oids = set.piped.as_ref().map_or(oids, |piped| &piped.ids);
    Ok(match &set.filter {
        Some(filter) => Selector::Filter(
            filter.clone(),
            set.place.clone(),
            set.project_filters.clone(),
        ),
        _ if oids.is_empty() => {
            return Err(CliError::Usage(
                "name the objects: give oids, - to read them from dam list, or --filter NAME"
                    .into(),
            ));
        }
        _ => Selector::Oids(
            oids.iter()
                .map(|text| ctx.dam().resolve_object(text))
                .collect::<Result<_, _>>()?,
        ),
    })
}

pub(crate) fn report_set(ctx: &Context, changes: &SetChanges) -> Result<Report, CliError> {
    report(&SetView::of(changes, ctx.config.owner_name()))
}

pub(crate) fn report_preview(
    ctx: &Context,
    written: Vec<Change>,
    sent: Option<SetChange>,
) -> Result<Report, CliError> {
    let changes: Vec<SetChange> = written
        .into_iter()
        .map(|change| SetChange { change, sent: None })
        .chain(sent)
        .collect();
    let previewed = SetChanges {
        preview: true,
        oids: changes.iter().map(|c| c.change.oid().clone()).collect(),
        changes,
    };
    report_set(ctx, &previewed)
}

mod confirm;
mod view;

#[cfg(test)]
mod tests;
