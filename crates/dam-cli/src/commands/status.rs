use dam_application::{CredentialSpec, Repositories, Unpushed, diff_staged, diff_working, status};
use dam_domain::Change;

use crate::args::{DiffArgs, StatusArgs};
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;

mod notices;
mod rows;

use notices::{notice_json, notice_line};
pub(crate) use rows::{change_json, change_line};
use rows::{conflict_json, conflict_line};

pub(crate) fn run_status(ctx: &mut Context, args: StatusArgs) -> Result<Report, CliError> {
    let remotes: Vec<_> = ctx.config.remotes.iter().map(|r| r.name.clone()).collect();
    let s = status(Repositories::of(ctx.store.as_ref()), &remotes)?;
    let mut human = Vec::new();
    section(&mut human, "Staged:", s.staged.iter().map(change_line));
    section(
        &mut human,
        "Not staged:",
        s.unstaged.iter().map(change_line),
    );
    section(
        &mut human,
        "Conflicts:",
        s.conflicts.iter().map(conflict_line),
    );
    section(&mut human, "Notices:", s.notices.iter().map(notice_line));
    section(
        &mut human,
        "Unpushed:",
        s.unpushed
            .iter()
            .filter(|u| !u.ids.is_empty())
            .map(|u| format!("  {}: {} commit(s)", u.remote.0, u.ids.len())),
    );
    if human.is_empty() {
        human.push("nothing staged, nothing changed".to_string());
    }
    if let Some(warning) = literal_credential_warning(ctx) {
        human.insert(0, warning);
    }
    let staged = change_documents(&s.staged, args.full)?;
    let unstaged = change_documents(&s.unstaged, args.full)?;
    let conflicts: Vec<serde_json::Value> = s
        .conflicts
        .iter()
        .map(conflict_json)
        .collect::<Result<_, _>>()?;
    Ok(Report {
        human: human.join("\n"),
        data: serde_json::json!({
            "staged": staged,
            "unstaged": unstaged,
            "conflicts": conflicts,
            "notices": s.notices.iter().map(notice_json).collect::<Vec<_>>(),
            "unpushed": s.unpushed.iter().map(unpushed_json).collect::<Vec<_>>(),
        }),
    })
}

pub(crate) fn run_diff(ctx: &mut Context, args: DiffArgs) -> Result<Report, CliError> {
    let changes = if args.staged {
        diff_staged(ctx.store.as_ref())?
    } else {
        diff_working(ctx.store.as_ref(), ctx.store.as_ref(), ctx.store.as_ref())?
    };
    let data = change_documents(&changes, args.full)?;
    Ok(Report {
        human: changes
            .iter()
            .map(change_line)
            .collect::<Vec<_>>()
            .join("\n"),
        data: serde_json::json!({ "changes": data }),
    })
}

/// One remote's owed commits: the ids in `dam log`'s order, and the count
/// beside them, which is the length of that list.
fn unpushed_json(unpushed: &Unpushed) -> serde_json::Value {
    serde_json::json!({
        "remote": unpushed.remote.0,
        "commits": unpushed.ids.len(),
        "oids": unpushed.ids.iter().map(|id| id.to_string()).collect::<Vec<_>>(),
    })
}

fn change_documents(changes: &[Change], full: bool) -> Result<Vec<serde_json::Value>, CliError> {
    changes.iter().map(|c| change_json(c, full)).collect()
}

/// One line naming every credential held as a value in the config file, which
/// the design spec has `dam status` warn about.
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

/// Pushes a titled block of already-indented lines, skipping an empty section.
fn section(out: &mut Vec<String>, title: &str, lines: impl Iterator<Item = String>) {
    let lines: Vec<String> = lines.collect();
    if lines.is_empty() {
        return;
    }
    out.push(title.to_string());
    out.extend(lines.into_iter().map(|l| {
        if l.starts_with("  ") {
            l
        } else {
            format!("  {l}")
        }
    }));
}

#[cfg(test)]
mod tests;
