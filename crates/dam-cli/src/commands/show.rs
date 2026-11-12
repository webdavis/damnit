use dam_application::{Refusal, UseCaseError};
use dam_domain::{Object, Permission};
use serde::Serialize;

use crate::args::ShowArgs;
use crate::commands::commit::CommitDoc;
use crate::commands::remote::refresh_stale_unless_no_pull;
use crate::context::Context;
use crate::error::CliError;
use crate::gate::may;
use crate::layout::table;
use crate::output::{Report, object_cells};
use crate::view::{Human, View, report, wire};

pub(crate) fn run_show(ctx: &Context, args: ShowArgs) -> Result<Report, CliError> {
    refresh_stale_unless_no_pull(ctx)?;
    if let Some(object) = working_object_named(ctx, &args.id)? {
        return report(&ShownObject(&object, &ctx.tz));
    }
    if !may(ctx.access.as_ref(), Permission::HistoryRead) {
        return Err(Refusal::NoSuchObject(args.id).into());
    }
    let record = ctx.dam().resolve_commit(&args.id)?;
    report(&ShownCommit(CommitDoc::new(&record, &ctx.tz)))
}

fn working_object_named(ctx: &Context, text: &str) -> Result<Option<Object>, CliError> {
    match ctx.dam().resolve_object(text) {
        Ok(oid) => Ok(ctx.dam().seen_object(&oid)?),
        Err(UseCaseError::Refused(Refusal::NoWorkingObject(_))) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

#[derive(Serialize)]
#[serde(transparent)]
struct ShownObject<'a>(
    #[serde(serialize_with = "wire")] &'a Object,
    #[serde(skip)] &'a jiff::tz::TimeZone,
);

impl View for ShownObject<'_> {
    fn human(&self) -> Human {
        let b = self.0.base();
        let mut human = table(&[object_cells(self.0, self.1)]);
        human.push(format!("oid: {}", b.oid));
        if !b.body.is_empty() {
            human.push(String::new());
            human.push(b.body.clone());
        }
        if !b.depends.is_empty() {
            human.push(format!(
                "depends: {}",
                b.depends
                    .iter()
                    .map(|d| d.short())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        Human::Lines(human)
    }
}

#[derive(Serialize)]
#[serde(transparent)]
struct ShownCommit<'a>(CommitDoc<'a>);

impl View for ShownCommit<'_> {
    fn human(&self) -> Human {
        let mut human = table(&[self.0.cells()]);
        human.extend(self.0.changes.iter().map(|c| format!("  {}", c.line())));
        Human::Lines(human)
    }
}

#[cfg(test)]
mod tests;
