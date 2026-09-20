use dam_application::{PullReport, PushReport, Repositories, Side, pull, push, resolve};

use crate::args::{PullArgs, PushArgs, ResolveArgs};
use crate::context::Context;
use crate::error::CliError;
use crate::oids::resolve_oid;
use crate::output::Report;

pub(crate) fn run_push(ctx: &mut Context, args: PushArgs) -> Result<Report, CliError> {
    let reports = push(
        Repositories::of(ctx.store.as_ref()),
        ctx.launcher.as_ref(),
        ctx.credentials.as_ref(),
        &ctx.config,
        args.remote.as_deref(),
    )?;
    Ok(Report {
        human: reports.iter().map(push_line).collect::<Vec<_>>().join("\n"),
        data: serde_json::json!({ "remotes": reports.iter().map(push_json).collect::<Vec<_>>() }),
    })
}

pub(crate) fn run_pull(ctx: &mut Context, args: PullArgs) -> Result<Report, CliError> {
    let reports = pull(
        Repositories::of(ctx.store.as_ref()),
        ctx.launcher.as_ref(),
        ctx.credentials.as_ref(),
        ctx.clock.as_ref(),
        ctx.random.as_mut(),
        &ctx.config,
        args.remote.as_deref(),
    )?;
    Ok(Report {
        human: reports.iter().map(pull_line).collect::<Vec<_>>().join("\n"),
        data: serde_json::json!({ "remotes": reports.iter().map(pull_json).collect::<Vec<_>>() }),
    })
}

pub(crate) fn run_resolve(ctx: &mut Context, args: ResolveArgs) -> Result<Report, CliError> {
    let oid = resolve_oid(ctx.store.as_ref(), &args.oid)?;
    let side = if args.theirs {
        Side::Theirs
    } else {
        Side::Ours
    };
    resolve(
        ctx.store.as_ref(),
        ctx.store.as_ref(),
        ctx.store.as_ref(),
        ctx.clock.as_ref(),
        ctx.random.as_mut(),
        &oid,
        side,
    )?;
    let word = if args.theirs { "theirs" } else { "ours" };
    Ok(Report {
        human: format!("{} resolved with {word}", oid.short()),
        data: serde_json::json!({ "oid": oid.to_string(), "side": word }),
    })
}

fn push_line(r: &PushReport) -> String {
    let mut line = format!(
        "{}: {} sent, {} ok, {} failed, {} skipped",
        r.remote.0,
        r.sent,
        r.succeeded,
        r.failed.len(),
        r.skipped
    );
    for (oid, why) in &r.failed {
        line.push_str(&format!("\n  {}: {why}", oid.short()));
    }
    line
}

fn push_json(r: &PushReport) -> serde_json::Value {
    serde_json::json!({
        "remote": r.remote.0,
        "sent": r.sent,
        "succeeded": r.succeeded,
        "skipped": r.skipped,
        "failed": r.failed.iter().map(|(o, w)| serde_json::json!({ "oid": o.to_string(), "why": w })).collect::<Vec<_>>(),
    })
}

fn pull_line(r: &PullReport) -> String {
    format!(
        "{}: {} new, {} updated, {} unchanged, {} conflict(s), {} removed upstream",
        r.remote.0, r.created, r.updated, r.unchanged, r.conflicts, r.removed_upstream
    )
}

fn pull_json(r: &PullReport) -> serde_json::Value {
    serde_json::json!({
        "remote": r.remote.0,
        "created": r.created,
        "updated": r.updated,
        "unchanged": r.unchanged,
        "conflicts": r.conflicts,
        "removed_upstream": r.removed_upstream,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::{AddArgs, CommitArgs, PullArgs, PushArgs, ResolveArgs};
    use crate::commands::commit::run_commit;
    use crate::commands::stage::run_add;
    use crate::testing::{EchoLauncher, context};
    use dam_application::{IncomingObject, PullOutcome};
    use dam_application::{RemoteConfig, RemoteName};
    use dam_domain::{Object, Oid, Task};
    use std::cell::RefCell;

    fn with_remote() -> Context {
        let mut ctx = context();
        ctx.config.remotes.push(RemoteConfig {
            name: RemoteName("t".into()),
            helper: "t".into(),
            url: "t::".into(),
            credentials: vec![],
            stale: None,
            deadline: None,
            path: None,
        });
        ctx
    }

    #[test]
    fn push_sends_unpushed_commits_and_reports_counts() {
        let mut ctx = with_remote();
        ctx.store
            .put(&Object::Task(Task::new(
                Oid::generate(&mut |x: &mut [u8]| x.fill(1)),
                "a",
            )))
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
        let report = run_push(&mut ctx, PushArgs { remote: None }).unwrap();
        assert!(report.human.contains("t: 1 sent, 1 ok"), "{}", report.human);
        assert_eq!(report.data["remotes"][0]["succeeded"], 1);
        assert!(
            ctx.store
                .unpushed(&RemoteName("t".into()))
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn pull_reports_what_arrived_and_resolve_settles_a_conflict() {
        let mut ctx = with_remote();
        let oid = Oid::generate(&mut |x: &mut [u8]| x.fill(1));
        ctx.store
            .put(&Object::Task(Task::new(oid.clone(), "mine")))
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
        run_push(&mut ctx, PushArgs { remote: None }).unwrap();
        let mut mine = Task::new(oid.clone(), "mine, edited");
        mine.base.subject = "mine, edited".into();
        ctx.store.put(&Object::Task(mine)).unwrap();
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
                message: "edit".into(),
            },
        )
        .unwrap();
        let theirs = IncomingObject {
            remote_id: format!("r-{}", &oid.as_str()[..4]),
            object: Object::Task(Task::new(oid.clone(), "theirs")),
        };
        ctx.launcher = Box::new(EchoLauncher {
            pulled: RefCell::new(Some(PullOutcome {
                objects: vec![theirs],
                ..PullOutcome::default()
            })),
        });
        let report = run_pull(
            &mut ctx,
            PullArgs {
                remote: Some("t".into()),
            },
        )
        .unwrap();
        assert_eq!(report.data["remotes"][0]["conflicts"], 1);
        assert_eq!(ctx.store.conflicts().unwrap().len(), 1);
        run_resolve(
            &mut ctx,
            ResolveArgs {
                oid: oid.short().into(),
                ours: false,
                theirs: true,
            },
        )
        .unwrap();
        assert!(ctx.store.conflicts().unwrap().is_empty());
        assert_eq!(
            ctx.store.get(&oid).unwrap().unwrap().base().subject,
            "theirs"
        );
    }
}
