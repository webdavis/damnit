use dam_application::Standing;
use dam_domain::Permission::{ConfigRead, ConflictRead, RemoteRead};
use dam_domain::{Authored, Change};

use crate::args::{DiffArgs, StatusArgs};
use crate::context::Context;
use crate::error::CliError;
use crate::gate::may;
use crate::output::Report;
use crate::view::report;

mod notices;
mod rows;
mod view;
mod waiting;

use notices::NoticeRow;
use rows::ConflictRow;
pub(crate) use rows::{ChangeRow, the_owner_to};
use view::{DiffView, StatusView, UnpushedRow};
use waiting::ItemRow;

pub(crate) fn run_status(ctx: &Context, args: StatusArgs) -> Result<Report, CliError> {
    let remotes: Vec<_> = ctx.config.remotes.iter().map(|r| r.name.clone()).collect();
    let s = ctx.dam().status(&remotes)?;
    let shown = |permission| may(ctx.access.as_ref(), permission);
    let warnings = token_warnings(ctx);
    report(&StatusView {
        staged: authored_rows(&s.staged, args.full).collect(),
        unstaged: owner_rows(&s.unstaged, args.full).collect(),
        waiting: s
            .waiting
            .iter()
            .map(|item| ItemRow::new(item, ctx.clock.now(), args.full))
            .collect(),
        conflicts: only_if(
            shown(ConflictRead),
            s.conflicts.iter().map(ConflictRow::new),
        ),
        notices: only_if(shown(RemoteRead), s.notices.iter().map(NoticeRow::new)),
        unpushed: only_if(shown(RemoteRead), s.unpushed.iter().map(UnpushedRow::new)),
        warnings: only_if(shown(ConfigRead), warnings),
        owner: the_owner_to(ctx.access.as_ref()),
    })
}

fn only_if<T>(shown: bool, rows: impl Iterator<Item = T>) -> Vec<T> {
    if shown { rows.collect() } else { Vec::new() }
}

pub(crate) fn run_diff(ctx: &Context, args: DiffArgs) -> Result<Report, CliError> {
    if args.staged {
        let staged = ctx.dam().seen_staged()?;
        return report(&DiffView {
            changes: authored_rows(&staged, args.full).collect(),
            owner: the_owner_to(ctx.access.as_ref()),
        });
    }
    let working = ctx.dam().diff_working()?;
    report(&DiffView {
        changes: owner_rows(&working, args.full).collect(),
        owner: the_owner_to(ctx.access.as_ref()),
    })
}

fn authored_rows(changes: &[Authored], full: bool) -> impl Iterator<Item = ChangeRow<'_>> {
    changes
        .iter()
        .map(move |c| ChangeRow::new(&c.change, c.author.token(), full))
}

fn owner_rows(changes: &[Change], full: bool) -> impl Iterator<Item = ChangeRow<'_>> {
    changes.iter().map(move |c| ChangeRow::new(c, None, full))
}

fn token_warnings(ctx: &Context) -> impl Iterator<Item = String> + '_ {
    let tokens = &ctx.config.tokens;
    let skipped = tokens.warnings.iter().map(|w| format!("warning: {w}"));
    let disabled = tokens.entries.iter().filter_map(|t| match &t.standing {
        Standing::Disabled(why) => Some(format!("warning: token {} is disabled: {why}", t.name)),
        Standing::Usable(_) => None,
    });
    skipped.chain(disabled)
}

#[cfg(test)]
mod skipped_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod token_section_tests;
#[cfg(test)]
mod token_warning_tests;
#[cfg(test)]
mod warning_tests;
#[cfg(test)]
mod width_tests;
