use dam_application::{Conflict, CredentialSpec, Notice, diff_staged, diff_working, status};
use dam_domain::{Change, Op, changed_fields};

use crate::args::DiffArgs;
use crate::context::Context;
use crate::error::CliError;
use crate::output::{Report, object_json};

pub fn run_status(ctx: &mut Context) -> Result<Report, CliError> {
    let remotes: Vec<_> = ctx.config.remotes.iter().map(|r| r.name.clone()).collect();
    let s = status(ctx.store.as_ref(), &remotes)?;
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
    Ok(Report {
        human: human.join("\n"),
        data: serde_json::json!({
            "staged": s.staged.iter().map(change_json).collect::<Vec<_>>(),
            "unstaged": s.unstaged.iter().map(change_json).collect::<Vec<_>>(),
            "conflicts": s.conflicts.iter().map(conflict_json).collect::<Vec<_>>(),
            "notices": s.notices.iter().map(notice_json).collect::<Vec<_>>(),
            "unpushed": s.unpushed.iter().map(|(r, n)| serde_json::json!({ "remote": r.0, "commits": n })).collect::<Vec<_>>(),
        }),
    })
}

pub fn run_diff(ctx: &mut Context, args: DiffArgs) -> Result<Report, CliError> {
    let changes = if args.staged {
        diff_staged(ctx.store.as_ref())?
    } else {
        diff_working(ctx.store.as_ref())?
    };
    Ok(Report {
        human: changes
            .iter()
            .map(change_line)
            .collect::<Vec<_>>()
            .join("\n"),
        data: serde_json::json!({ "changes": changes.iter().map(change_json).collect::<Vec<_>>() }),
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

pub fn change_line(change: &Change) -> String {
    let subject = change
        .after
        .as_ref()
        .or(change.before.as_ref())
        .map(|o| o.base().subject.as_str())
        .unwrap_or("");
    match change.op {
        Op::Create => format!("new      {}  {subject}", change.oid.short()),
        Op::Delete => format!("removed  {}  {subject}", change.oid.short()),
        Op::Update => {
            let fields = match (&change.before, &change.after) {
                (Some(b), Some(a)) => changed_fields(b, a).join(", "),
                _ => String::new(),
            };
            format!("changed  {}  {subject}  ({fields})", change.oid.short())
        }
    }
}

pub fn change_json(change: &Change) -> serde_json::Value {
    serde_json::json!({
        "oid": change.oid.to_string(),
        "op": match change.op { Op::Create => "create", Op::Update => "update", Op::Delete => "delete" },
        "before": change.before.as_ref().map(object_json),
        "after": change.after.as_ref().map(object_json),
    })
}

fn conflict_line(c: &Conflict) -> String {
    format!(
        "{}  {}  ours: {:?}  theirs: {:?}  (dam resolve {} --ours|--theirs)",
        c.oid.short(),
        c.remote.0,
        c.ours.base().subject,
        c.theirs.base().subject,
        c.oid.short()
    )
}

fn conflict_json(c: &Conflict) -> serde_json::Value {
    serde_json::json!({ "oid": c.oid.to_string(), "remote": c.remote.0, "ours": object_json(&c.ours), "theirs": object_json(&c.theirs) })
}

fn notice_line(n: &Notice) -> String {
    match n {
        Notice::RemovedUpstream {
            remote,
            oid,
            subject,
        } => format!(
            "{} was removed on {}: {subject:?} is kept here; dam rm {} to drop it",
            oid.short(),
            remote.0,
            oid.short()
        ),
        Notice::EventCancelled {
            oid,
            subject,
            attached,
        } => format!(
            "event {} {subject:?} was cancelled; {attached} attached task(s) kept",
            oid.short()
        ),
        Notice::PushFailed { remote, oid, why } => {
            format!("push of {} to {} failed: {why}", oid.short(), remote.0)
        }
        Notice::PullFailed { remote, why } => {
            format!("pull from {} failed: {why}", remote.0)
        }
        Notice::KindChanged { oid, ours, theirs } => format!(
            "{} is {} {ours} here and {} {theirs} upstream",
            oid.short(),
            article(ours),
            article(theirs)
        ),
    }
}

/// The article a kind word takes, so a reworded or added kind still reads.
fn article(word: &str) -> &'static str {
    match word.chars().next() {
        Some('a' | 'e' | 'i' | 'o' | 'u') => "an",
        _ => "a",
    }
}

fn notice_json(n: &Notice) -> serde_json::Value {
    match n {
        Notice::RemovedUpstream {
            remote,
            oid,
            subject,
        } => {
            serde_json::json!({ "kind": "removed_upstream", "remote": remote.0, "oid": oid.to_string(), "subject": subject })
        }
        Notice::EventCancelled {
            oid,
            subject,
            attached,
        } => {
            serde_json::json!({ "kind": "event_cancelled", "oid": oid.to_string(), "subject": subject, "attached": attached })
        }
        Notice::PushFailed { remote, oid, why } => {
            serde_json::json!({ "kind": "push_failed", "remote": remote.0, "oid": oid.to_string(), "why": why })
        }
        Notice::PullFailed { remote, why } => {
            serde_json::json!({ "kind": "pull_failed", "remote": remote.0, "why": why })
        }
        Notice::KindChanged { oid, ours, theirs } => {
            serde_json::json!({ "kind": "kind_changed", "oid": oid.to_string(), "ours": ours, "theirs": theirs })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::args::{AddArgs, CommitArgs, DiffArgs};
    use crate::commands::commit::run_commit;
    use crate::commands::stage::run_add;
    use crate::testing::context;
    use dam_application::{CredentialSpec, RemoteConfig, RemoteName};
    use dam_domain::{Object, Oid, Task};

    fn oid(b: u8) -> Oid {
        Oid::generate(&mut |x: &mut [u8]| x.fill(b))
    }

    #[test]
    fn status_separates_staged_unstaged_and_notices() {
        let mut ctx = context();
        ctx.store
            .put(&Object::Task(Task::new(oid(1), "staged")))
            .unwrap();
        run_add(
            &mut ctx,
            AddArgs {
                oids: vec![],
                all: true,
            },
        )
        .unwrap();
        ctx.store
            .put(&Object::Task(Task::new(oid(2), "loose")))
            .unwrap();
        ctx.store
            .add_notice(&dam_application::Notice::PushFailed {
                remote: dam_application::RemoteName("t".into()),
                oid: oid(1),
                why: "nope".into(),
            })
            .unwrap();
        let report = super::run_status(&mut ctx).unwrap();
        assert!(report.human.contains("Staged:"));
        assert!(report.human.contains("Not staged:"));
        assert!(report.human.contains("Notices:"));
        assert!(report.human.contains("nope"));
        assert_eq!(report.data["staged"].as_array().unwrap().len(), 1);
        assert_eq!(report.data["unstaged"].as_array().unwrap().len(), 1);
        assert_eq!(report.data["notices"].as_array().unwrap().len(), 1);
    }

    fn remote_with(credentials: Vec<CredentialSpec>) -> RemoteConfig {
        RemoteConfig {
            name: RemoteName("todoist".into()),
            helper: "todoist".into(),
            url: "todoist::".into(),
            credentials,
            stale: None,
            deadline: None,
            path: None,
        }
    }

    #[test]
    fn status_warns_once_when_a_credential_is_a_value_in_the_config_file() {
        let mut ctx = context();
        ctx.config.remotes = vec![remote_with(vec![CredentialSpec::Literal {
            name: "api_token".into(),
            value: "SUPERSECRETTOKEN".into(),
        }])];
        let report = super::run_status(&mut ctx).unwrap();
        let warnings: Vec<&str> = report
            .human
            .lines()
            .filter(|l| l.starts_with("warning:"))
            .collect();
        assert_eq!(warnings.len(), 1, "{}", report.human);
        assert!(warnings[0].contains("todoist.api_token"), "{}", warnings[0]);
        assert!(!report.human.contains("SUPERSECRETTOKEN"));
    }

    #[test]
    fn status_does_not_warn_when_no_credential_is_a_value_in_the_config_file() {
        let mut ctx = context();
        ctx.config.remotes = vec![remote_with(vec![CredentialSpec::Env {
            name: "api_token".into(),
            var: "DAM_TODOIST_API_TOKEN".into(),
        }])];
        let report = super::run_status(&mut ctx).unwrap();
        assert!(!report.human.contains("warning:"), "{}", report.human);
    }

    #[test]
    fn a_kind_change_notice_names_both_kinds() {
        let mut ctx = context();
        ctx.store
            .add_notice(&dam_application::Notice::KindChanged {
                oid: oid(1),
                ours: "task".into(),
                theirs: "event".into(),
            })
            .unwrap();
        let report = super::run_status(&mut ctx).unwrap();
        assert!(
            report.human.contains(&format!(
                "{} is a task here and an event upstream",
                oid(1).short()
            )),
            "{}",
            report.human
        );
        assert_eq!(report.data["notices"][0]["kind"], "kind_changed");
        assert_eq!(report.data["notices"][0]["ours"], "task");
        assert_eq!(report.data["notices"][0]["theirs"], "event");
    }

    #[test]
    fn a_kind_change_notice_reads_the_other_way_round_too() {
        let mut ctx = context();
        ctx.store
            .add_notice(&dam_application::Notice::KindChanged {
                oid: oid(1),
                ours: "event".into(),
                theirs: "task".into(),
            })
            .unwrap();
        let report = super::run_status(&mut ctx).unwrap();
        assert!(
            report.human.contains(&format!(
                "{} is an event here and a task upstream",
                oid(1).short()
            )),
            "{}",
            report.human
        );
    }

    #[test]
    fn diff_shows_working_or_staged() {
        let mut ctx = context();
        ctx.store
            .put(&Object::Task(Task::new(oid(1), "a")))
            .unwrap();
        run_add(
            &mut ctx,
            AddArgs {
                oids: vec![],
                all: true,
            },
        )
        .unwrap();
        run_commit(
            &mut ctx,
            CommitArgs {
                message: "m".into(),
            },
        )
        .unwrap();
        let mut t = Task::new(oid(1), "b");
        t.base.subject = "b".into();
        ctx.store.put(&Object::Task(t)).unwrap();
        let working = super::run_diff(&mut ctx, DiffArgs { staged: false }).unwrap();
        assert!(working.human.contains("changed"));
        assert!(working.human.contains("subject"));
        let staged = super::run_diff(&mut ctx, DiffArgs { staged: true }).unwrap();
        assert!(staged.human.is_empty());
    }

    #[test]
    fn status_reports_conflicts_and_unpushed_commits() {
        let mut ctx = context();
        ctx.config.remotes.push(RemoteConfig {
            name: RemoteName("todoist".into()),
            helper: "todoist".into(),
            url: "todoist::".into(),
            credentials: vec![],
            stale: None,
            deadline: None,
            path: None,
        });
        ctx.store
            .put(&Object::Task(Task::new(oid(1), "mine")))
            .unwrap();
        ctx.store
            .mark_conflict(
                &RemoteName("todoist".into()),
                &oid(1),
                &Object::Task(Task::new(oid(1), "theirs")),
            )
            .unwrap();
        ctx.store
            .put(&Object::Task(Task::new(oid(2), "b")))
            .unwrap();
        run_add(
            &mut ctx,
            AddArgs {
                oids: vec![],
                all: true,
            },
        )
        .unwrap();
        run_commit(
            &mut ctx,
            CommitArgs {
                message: "m".into(),
            },
        )
        .unwrap();
        let report = super::run_status(&mut ctx).unwrap();
        assert!(report.human.contains("Conflicts:"));
        assert!(report.human.contains(oid(1).short()));
        assert!(report.human.contains("\"mine\""));
        assert!(report.human.contains("\"theirs\""));
        assert!(report.human.contains("Unpushed:"));
        assert!(report.human.contains("todoist: 1 commit(s)"));
        assert_eq!(report.data["conflicts"].as_array().unwrap().len(), 1);
        assert_eq!(report.data["conflicts"][0]["oid"], oid(1).to_string());
        assert_eq!(report.data["unpushed"][0]["remote"], "todoist");
        assert_eq!(report.data["unpushed"][0]["commits"], 1);
    }
}
