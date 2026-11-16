use dam_application::{Named, Reopened};
use dam_domain::{ItemHash, Oid};

use self::removals::AddedRemovals;
use crate::args::{AddArgs, ResetArgs};
use crate::commands::hooked::hooked;
use crate::commands::status::ChangeRow;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{Human, View, displays, report};
use serde::Serialize;

#[derive(Serialize)]
struct Staged<'a> {
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    preview: bool,
    staged: Vec<ChangeRow<'a>>,
    #[serde(serialize_with = "displays", skip_serializing_if = "<[_]>::is_empty")]
    taken: &'a [ItemHash],
    #[serde(flatten)]
    removals: AddedRemovals<'a>,
}

impl View for Staged<'_> {
    fn human(&self) -> Human {
        let staged = self.staged.iter().map(ChangeRow::line);
        let mut lines: Vec<String> = staged.chain(self.removals.lines()).collect();
        if self.preview {
            lines.extend(
                self.taken
                    .iter()
                    .map(|hash| format!("would take {}", hash.short())),
            );
            lines.push(format!(
                "preview: {} would be staged, {} waiting items taken, nothing written",
                self.staged.len(),
                self.taken.len()
            ));
        }
        Human::list(
            lines.into_iter().map(|line| vec![line]).collect(),
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
    #[serde(serialize_with = "displays", skip_serializing_if = "<[_]>::is_empty")]
    kept: &'a [Oid],
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
        if !self.kept.is_empty() {
            let oids = self.kept.iter().map(Oid::short).collect();
            lines.push(format!("kept {}, changed since dam add", shorts(oids)));
        }
        Human::Lines(lines)
    }
}

pub(crate) fn run_add(ctx: &Context, args: AddArgs) -> Result<Report, CliError> {
    let (added, taken) = adding::added(ctx, &args)?;
    let staged = Staged {
        preview: args.preview,
        taken: &taken,
        staged: added
            .staged
            .iter()
            .map(|c| ChangeRow::authored(ctx, c, false))
            .collect(),
        removals: AddedRemovals::new(&added.removals),
    };
    report(&hooked(ctx, staged))
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
            Named::Set(set) => {
                return Err(CliError::Usage(format!(
                    "{} is a set still waiting; dam add or dam rm it",
                    set.short()
                )));
            }
        }
    }
    let mut kept = Vec::new();
    if !oids.is_empty() || hashes.is_empty() {
        kept = ctx.dam().reset(&oids)?;
    }
    let mut waiting = Vec::new();
    for hash in &hashes {
        match ctx.dam().reopen(hash)? {
            Reopened::Unstaged { oid, kept: more } => {
                oids.push(oid);
                kept.extend(more);
            }
            Reopened::Waiting(hash) => waiting.push(hash),
        }
    }
    report(&Unstaged {
        unstaged: &oids,
        waiting: &waiting,
        kept: &kept,
    })
}

mod adding;
mod removals;

#[cfg(test)]
mod preview_tests;
#[cfg(test)]
mod tests;
