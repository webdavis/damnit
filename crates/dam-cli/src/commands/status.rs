use dam_application::{CredentialSpec, Standing};
use dam_domain::Change;
use dam_domain::Permission::{ConfigRead, ConflictRead, RemoteRead};

use crate::args::{DiffArgs, StatusArgs};
use crate::context::Context;
use crate::error::CliError;
use crate::gate::may;
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
    let s = ctx.dam().status(&remotes)?;
    let shown = |permission| may(ctx.access.as_ref(), permission);
    let warnings = literal_credential_warning(ctx)
        .into_iter()
        .chain(token_warnings(ctx));
    report(&StatusView {
        staged: change_rows(&s.staged, args.full),
        unstaged: change_rows(&s.unstaged, args.full),
        conflicts: only_if(
            shown(ConflictRead),
            s.conflicts.iter().map(ConflictRow::new),
        ),
        notices: only_if(shown(RemoteRead), s.notices.iter().map(NoticeRow::new)),
        unpushed: only_if(shown(RemoteRead), s.unpushed.iter().map(UnpushedRow::new)),
        warnings: only_if(shown(ConfigRead), warnings),
    })
}

fn only_if<T>(shown: bool, rows: impl Iterator<Item = T>) -> Vec<T> {
    if shown { rows.collect() } else { Vec::new() }
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

fn token_warnings(ctx: &Context) -> impl Iterator<Item = String> + '_ {
    let tokens = &ctx.config.tokens;
    let skipped = tokens.warnings.iter().map(|w| format!("warning: {w}"));
    let disabled = tokens.entries.iter().filter_map(|t| match &t.standing {
        Standing::Disabled(why) => Some(format!("warning: token {} is disabled: {why}", t.name)),
        Standing::Usable(_) => None,
    });
    skipped.chain(disabled)
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
mod token_section_tests;
#[cfg(test)]
mod token_warning_tests;
#[cfg(test)]
mod warning_tests;
#[cfg(test)]
mod width_tests;
