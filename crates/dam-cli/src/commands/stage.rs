use dam_domain::Oid;

use self::removals::AddedRemovals;
use crate::args::{AddArgs, ResetArgs};
use crate::commands::status::ChangeRow;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{Human, View, displays, report};
use serde::Serialize;

#[derive(Serialize)]
struct Staged<'a> {
    staged: Vec<ChangeRow<'a>>,
    #[serde(flatten)]
    removals: AddedRemovals<'a>,
}

impl View for Staged<'_> {
    fn human(&self) -> Human {
        let staged = self.staged.iter().map(ChangeRow::line);
        Human::list(
            staged
                .chain(self.removals.lines())
                .map(|line| vec![line])
                .collect(),
            "nothing to stage",
        )
    }
}

#[derive(Serialize)]
struct Unstaged<'a> {
    #[serde(serialize_with = "displays")]
    unstaged: &'a [Oid],
}

impl View for Unstaged<'_> {
    fn human(&self) -> Human {
        Human::Lines(vec![match self.unstaged {
            [] => "unstaged everything".to_string(),
            oids => format!(
                "unstaged {}",
                oids.iter().map(Oid::short).collect::<Vec<_>>().join(", ")
            ),
        }])
    }
}

pub(crate) fn run_add(ctx: &Context, args: AddArgs) -> Result<Report, CliError> {
    let added = if args.all {
        ctx.dam().add_all()?
    } else {
        let named = args
            .oids
            .iter()
            .map(|t| ctx.dam().resolve_named(t))
            .collect::<Result<Vec<_>, _>>()?;
        ctx.dam().add(&named)?
    };
    report(&Staged {
        staged: added
            .staged
            .iter()
            .map(|c| ChangeRow::authored(ctx, c, false))
            .collect(),
        removals: AddedRemovals::new(&added.removals),
    })
}

pub(crate) fn run_reset(ctx: &Context, args: ResetArgs) -> Result<Report, CliError> {
    let oids = args
        .oids
        .iter()
        .map(|t| ctx.dam().resolve_object(t))
        .collect::<Result<Vec<Oid>, _>>()?;
    ctx.dam().reset(&oids)?;
    report(&Unstaged { unstaged: &oids })
}

mod removals;

#[cfg(test)]
mod tests;
