use dam_application::{Named, Reopened};
use dam_domain::{ItemHash, Oid};

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
    #[serde(serialize_with = "displays", skip_serializing_if = "<[_]>::is_empty")]
    waiting: &'a [ItemHash],
}

impl View for Unstaged<'_> {
    fn human(&self) -> Human {
        let shorts = |shorts: Vec<&str>| shorts.join(", ");
        let mut lines = Vec::new();
        if !self.unstaged.is_empty() || self.waiting.is_empty() {
            lines.push(match self.unstaged {
                [] => "unstaged everything".to_string(),
                oids => format!("unstaged {}", shorts(oids.iter().map(Oid::short).collect())),
            });
        }
        if !self.waiting.is_empty() {
            let hashes = self.waiting.iter().map(ItemHash::short).collect();
            lines.push(format!("waiting again {}", shorts(hashes)));
        }
        Human::Lines(lines)
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
    let named = args
        .oids
        .iter()
        .map(|t| ctx.dam().resolve_resettable(t))
        .collect::<Result<Vec<Named>, _>>()?;
    let mut oids = Vec::new();
    let mut hashes = Vec::new();
    for name in named {
        match name {
            Named::Object(oid) => oids.push(oid),
            Named::Item(hash) => hashes.push(hash),
        }
    }
    if !oids.is_empty() || hashes.is_empty() {
        ctx.dam().reset(&oids)?;
    }
    let mut waiting = Vec::new();
    for hash in &hashes {
        match ctx.dam().reopen(hash)? {
            Reopened::Unstaged(oid) => oids.push(oid),
            Reopened::Waiting(hash) => waiting.push(hash),
        }
    }
    report(&Unstaged {
        unstaged: &oids,
        waiting: &waiting,
    })
}

mod removals;

#[cfg(test)]
mod tests;
