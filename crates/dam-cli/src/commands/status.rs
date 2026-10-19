use dam_application::{CredentialSpec, Repositories, status};
use dam_domain::Change;

use crate::args::{DiffArgs, StatusArgs};
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::report;

mod notices;
mod rows;
mod view;

use notices::NoticeRow;
pub(crate) use rows::ChangeRow;
use rows::ConflictRow;
use view::{DiffView, StatusView, UnpushedRow};

pub(crate) fn run_status(ctx: &Context, args: StatusArgs) -> Result<Report, CliError> {
    let remotes: Vec<_> = ctx.config.remotes.iter().map(|r| r.name.clone()).collect();
    let s = status(Repositories::of(&*ctx.store), &remotes)?;
    report(&StatusView {
        staged: change_rows(&s.staged, args.full),
        unstaged: change_rows(&s.unstaged, args.full),
        conflicts: s.conflicts.iter().map(ConflictRow::new).collect(),
        notices: s.notices.iter().map(NoticeRow::new).collect(),
        unpushed: s.unpushed.iter().map(UnpushedRow::new).collect(),
        warnings: Vec::from_iter(literal_credential_warning(ctx)),
    })
}

pub(crate) fn run_diff(ctx: &Context, args: DiffArgs) -> Result<Report, CliError> {
    let changes = if args.staged {
        ctx.store.staged()?
    } else {
        ctx.dam().diff_working()?
    };
    report(&DiffView {
        changes: change_rows(&changes, args.full),
    })
}

fn change_rows(changes: &[Change], full: bool) -> Vec<ChangeRow<'_>> {
    changes.iter().map(|c| ChangeRow::new(c, full)).collect()
}

fn literal_credential_warning(ctx: &Context) -> Option<String> {
    let literals: Vec<String> = ctx
        .config
        .remotes
        .iter()
        .flat_map(|r| {
            r.credentials.iter().filter_map(move |c| match c {
                CredentialSpec::Literal { name, .. } => Some(format!("{}.{name}", r.name.0)),
                _ => None,
            })
        })
        .collect();
    if literals.is_empty() {
        return None;
    }
    Some(format!(
        "warning: a credential is a value in the config file: {}; prefer <name>_command or <name>_env",
        literals.join(", ")
    ))
}

#[cfg(test)]
mod skipped_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod warning_tests;
