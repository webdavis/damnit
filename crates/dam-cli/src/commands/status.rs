use dam_application::{Conflict, Notice, diff_staged, diff_working, status};
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
    }
}

#[cfg(test)]
mod tests {
    use crate::args::{AddArgs, CommitArgs, DiffArgs};
    use crate::commands::commit::run_commit;
    use crate::commands::stage::run_add;
    use crate::testing::context;
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
}
