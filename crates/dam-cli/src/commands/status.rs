use dam_application::Notice;
use dam_domain::Permission::{ConfigRead, ConflictRead, RemoteRead};
use dam_domain::{Authored, Change};

use super::token_warnings::token_warnings;
use crate::args::{DiffArgs, StatusArgs};
use crate::context::Context;
use crate::error::CliError;
use crate::gate::may;
use crate::output::Report;
use crate::view::report;

mod notices;
mod rows;
mod sets;
mod view;
mod waiting;

use notices::NoticeRow;
use rows::ConflictRow;
pub(crate) use rows::{ChangeRow, DetailsView, the_owner_to, visible};
use sets::SetRow;
use view::{DiffView, StatusView, UnpushedRow};
pub(crate) use waiting::ItemRow;

pub(crate) fn run_status(ctx: &Context, args: StatusArgs) -> Result<Report, CliError> {
    let remotes: Vec<_> = ctx.config.remotes.iter().map(|r| r.name.clone()).collect();
    let s = ctx.dam().status(&remotes)?;
    let shown = |permission| may(ctx.access.as_ref(), permission);
    let owners = ctx.access.token().is_none();
    let now = ctx.clock.now();
    let row = |item| {
        ItemRow::new(item, now, args.full)
            .told(ctx, item)
            .removing(s.removal_plans.get(&item.hash))
    };
    let sets = ctx.dam().waiting_sets()?;
    let in_set = |set| -> Vec<_> {
        s.waiting
            .iter()
            .filter(|item| item.set.as_ref() == Some(set))
            .collect()
    };
    let warnings = token_warnings(&ctx.config.tokens).map(|w| format!("warning: {w}"));
    report(&StatusView {
        staged: authored_rows(ctx, &s.staged, args.full).collect(),
        unstaged: owner_rows(ctx, &s.unstaged, args.full).collect(),
        waiting: s
            .waiting
            .iter()
            .filter(|item| item.set.is_none())
            .map(row)
            .collect(),
        sets: sets
            .iter()
            .filter_map(|set| {
                let members = in_set(set);
                let rows = members.iter().map(|item| row(item)).collect();
                SetRow::new(set, &members, rows, now)
            })
            .collect(),
        conflicts: only_if(
            shown(ConflictRead),
            s.conflicts.iter().map(ConflictRow::new),
        ),
        notices: only_if(
            shown(RemoteRead),
            s.notices
                .iter()
                .filter(|n| owners || !matches!(n, Notice::TaskPulledFailed { .. }))
                .map(NoticeRow::new),
        ),
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
            changes: authored_rows(ctx, &staged, args.full).collect(),
            owner: the_owner_to(ctx.access.as_ref()),
        });
    }
    let working = ctx.dam().diff_working()?;
    report(&DiffView {
        changes: owner_rows(ctx, &working, args.full).collect(),
        owner: the_owner_to(ctx.access.as_ref()),
    })
}

fn authored_rows<'a>(
    ctx: &'a Context,
    changes: &'a [Authored],
    full: bool,
) -> impl Iterator<Item = ChangeRow<'a>> {
    changes
        .iter()
        .map(move |c| ChangeRow::authored(ctx, c, full))
}

fn owner_rows<'a>(
    ctx: &'a Context,
    changes: &'a [Change],
    full: bool,
) -> impl Iterator<Item = ChangeRow<'a>> {
    let owner = ctx.config.owner_name();
    changes
        .iter()
        .map(move |c| ChangeRow::new(c, None, full).sent(owner, None))
}

#[cfg(test)]
mod missing_upstream_notice_tests;
#[cfg(test)]
mod notice_text_tests;
#[cfg(test)]
mod removed_conflict_tests;
#[cfg(test)]
mod removed_notice_tests;
#[cfg(test)]
mod skipped_tests;
#[cfg(test)]
mod task_pulled_notice_tests;
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
