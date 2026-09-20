use dam_application::{CredentialSpec, Repositories, diff_staged, diff_working, status};

use crate::args::DiffArgs;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;

mod notices;
mod rows;

use notices::{notice_json, notice_line};
pub(crate) use rows::{change_json, change_line};
use rows::{conflict_json, conflict_line};

pub(crate) fn run_status(ctx: &mut Context) -> Result<Report, CliError> {
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
            .filter(|(_, n)| *n > 0)
            .map(|(r, n)| format!("  {}: {n} commit(s)", r.0)),
    );
    if human.is_empty() {
        human.push("nothing staged, nothing changed".to_string());
    }
    if let Some(warning) = literal_credential_warning(ctx) {
        human.insert(0, warning);
    }
    let staged: Vec<serde_json::Value> =
        s.staged.iter().map(change_json).collect::<Result<_, _>>()?;
    let unstaged: Vec<serde_json::Value> = s
        .unstaged
        .iter()
        .map(change_json)
        .collect::<Result<_, _>>()?;
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
            "unpushed": s.unpushed.iter().map(|(r, n)| serde_json::json!({ "remote": r.0, "commits": n })).collect::<Vec<_>>(),
        }),
    })
}

pub(crate) fn run_diff(ctx: &mut Context, args: DiffArgs) -> Result<Report, CliError> {
    let changes = if args.staged {
        diff_staged(ctx.store.as_ref())?
    } else {
        diff_working(ctx.store.as_ref(), ctx.store.as_ref(), ctx.store.as_ref())?
    };
    let data: Vec<serde_json::Value> = changes.iter().map(change_json).collect::<Result<_, _>>()?;
    Ok(Report {
        human: changes
            .iter()
            .map(change_line)
            .collect::<Vec<_>>()
            .join("\n"),
        data: serde_json::json!({ "changes": data }),
    })
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
