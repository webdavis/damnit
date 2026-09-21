use dam_adapters::{append_remote, load_config};
use dam_application::{Notice, RemoteConfig, RemoteName, Repositories, pull};
use dam_domain::Timestamp;

use crate::args::{RemoteArgs, RemoteCommand};
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;

pub(crate) fn run_remote(ctx: &mut Context, args: RemoteArgs) -> Result<Report, CliError> {
    match args.command {
        RemoteCommand::Add { name, url } => {
            let warning = append_remote(&ctx.config_path, &name, &url)?;
            ctx.config = load_config(&ctx.config_path)?;
            let mut human = format!("added {name} ({url})");
            if let Some(why) = &warning {
                human.insert_str(0, &format!("warning: {why}\n"));
            }
            Ok(Report {
                human,
                data: serde_json::json!({
                    "name": name,
                    "url": url,
                    "warnings": Vec::from_iter(warning),
                }),
            })
        }
        RemoteCommand::List => list(ctx),
    }
}

/// Every remote with the times it was last reached, so a client header can
/// say how fresh what it is showing is.
fn list(ctx: &Context) -> Result<Report, CliError> {
    let now = ctx.clock.now();
    let mut lines = Vec::with_capacity(ctx.config.remotes.len());
    let mut rows = Vec::with_capacity(ctx.config.remotes.len());
    for r in &ctx.config.remotes {
        let synced = Synced {
            pulled: ctx.store.last_pull(&r.name)?,
            pushed: ctx.store.last_push(&r.name)?,
        };
        lines.push(remote_line(r, &synced, now));
        rows.push(remote_json(r, &synced));
    }
    Ok(Report {
        human: lines.join("\n"),
        data: serde_json::json!({ "remotes": rows }),
    })
}

/// When each sync verb last completed against one remote, `None` for one it
/// has never reached.
struct Synced {
    pulled: Option<Timestamp>,
    pushed: Option<Timestamp>,
}

fn remote_line(r: &RemoteConfig, synced: &Synced, now: Timestamp) -> String {
    let mut line = format!("{}  {}", r.name.0, r.url);
    if let Some(p) = &r.path {
        line.push_str(&format!("  path {}", p.as_str()));
    }
    if let Some(s) = r.stale {
        line.push_str(&format!("  stale {}s", s.as_secs()));
    }
    line.push_str(&format!("  {}", when_phrase("pulled", synced.pulled, now)));
    line.push_str(&format!("  {}", when_phrase("pushed", synced.pushed, now)));
    line
}

fn remote_json(r: &RemoteConfig, synced: &Synced) -> serde_json::Value {
    serde_json::json!({
        "name": r.name.0,
        "helper": r.helper,
        "url": r.url,
        "path": r.path.as_ref().map(|p| p.as_str()),
        "stale_seconds": r.stale.map(|s| s.as_secs()),
        "last_pull": synced.pulled.map(|t| t.to_string()),
        "last_push": synced.pushed.map(|t| t.to_string()),
    })
}

/// "pulled 4m ago", or "never pulled" for a remote that verb has not reached.
fn when_phrase(verb: &str, at: Option<Timestamp>, now: Timestamp) -> String {
    match at {
        None => format!("never {verb}"),
        Some(at) => format!("{verb} {}", ago(now, at)),
    }
}

/// How long ago `then` was, to one unit. A clock that has gone backwards
/// reads as the present rather than as the future.
fn ago(now: Timestamp, then: Timestamp) -> String {
    const MINUTE: i64 = 60;
    const HOUR: i64 = 60 * MINUTE;
    const DAY: i64 = 24 * HOUR;
    let seconds = now.duration_since(then).as_secs().max(0);
    if seconds < MINUTE {
        "just now".to_string()
    } else if seconds < HOUR {
        format!("{}m ago", seconds / MINUTE)
    } else if seconds < DAY {
        format!("{}h ago", seconds / HOUR)
    } else {
        format!("{}d ago", seconds / DAY)
    }
}

/// Pulls each remote whose `stale` is set and whose last pull is older than
/// it. A failure to reach a remote is a notice, not an error, so a read
/// never fails because the network did. `--no-pull` skips the whole pass, so
/// the read touches nothing but the store.
pub(crate) fn maybe_pull_stale(ctx: &mut Context) -> Result<(), CliError> {
    if ctx.no_pull {
        return Ok(());
    }
    let now = ctx.clock.now();
    let due: Vec<String> = ctx
        .config
        .remotes
        .iter()
        .filter_map(|r| {
            let stale = r.stale?;
            let last = ctx.store.last_pull(&r.name).ok().flatten();
            let age = last.map(|l| now.duration_since(l));
            let is_due = age
                .is_none_or(|a| a.as_secs() >= i64::try_from(stale.as_secs()).unwrap_or(i64::MAX));
            is_due.then(|| r.name.0.clone())
        })
        .collect();
    for name in due {
        if let Err(e) = pull(
            Repositories::of(ctx.store.as_ref()),
            ctx.launcher.as_ref(),
            ctx.credentials.as_ref(),
            ctx.clock.as_ref(),
            ctx.random.as_ref(),
            &ctx.config,
            Some(&name),
        ) {
            ctx.store.add_notice(&Notice::PullFailed {
                remote: RemoteName(name.clone()),
                why: e.to_string(),
            })?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::{RemoteArgs, RemoteCommand};
    use crate::testing::{EchoLauncher, FailingLauncher, context};
    use dam_application::{IncomingObject, PullOutcome};
    use dam_application::{RemoteConfig, RemoteName};
    use dam_domain::{Object, Oid, Task};
    use std::cell::RefCell;
    use std::time::Duration;

    /// One task as a remote sends it, under a remote id built from `tag`.
    fn incoming(tag: &str, subject: &str) -> IncomingObject {
        IncomingObject {
            remote_id: format!("r-{tag}"),
            object: Object::Task(Task::new(
                Oid::generate(&mut |b: &mut [u8]| b.fill(0)),
                subject,
            )),
        }
    }

    #[test]
    fn remote_add_writes_the_config_and_list_reads_it_back() {
        let dir = tempfile::tempdir().unwrap();
        let mut ctx = context();
        ctx.config_path = dir.path().join("config.toml");
        run_remote(
            &mut ctx,
            RemoteArgs {
                command: RemoteCommand::Add {
                    name: "todoist".into(),
                    url: "todoist::default".into(),
                },
            },
        )
        .unwrap();
        let report = run_remote(
            &mut ctx,
            RemoteArgs {
                command: RemoteCommand::List,
            },
        )
        .unwrap();
        assert!(
            report.human.contains("todoist::default"),
            "list must echo the url as configured, not just the helper prefix: {}",
            report.human
        );
        assert_eq!(ctx.config.remote("todoist").unwrap().helper, "todoist");
    }

    /// The times a header renders: RFC 3339 in the document, a phrase in the
    /// human line, and null for a remote neither verb has reached yet.
    #[test]
    fn remote_list_reports_the_last_pull_and_the_last_push() {
        let mut ctx = context();
        let t = RemoteName("t".into());
        ctx.config.remotes.push(RemoteConfig {
            name: t.clone(),
            helper: "t".into(),
            url: "t::".into(),
            credentials: vec![],
            stale: None,
            deadline: None,
            path: None,
        });
        let never = listed(&mut ctx);
        assert_eq!(
            never.data["remotes"][0]["last_pull"],
            serde_json::Value::Null
        );
        assert_eq!(
            never.data["remotes"][0]["last_push"],
            serde_json::Value::Null
        );
        assert!(never.human.contains("never pulled"), "{}", never.human);
        assert!(never.human.contains("never pushed"), "{}", never.human);

        let now = ctx.clock.now();
        ctx.store
            .set_last_pull(&t, now - std::time::Duration::from_secs(240))
            .unwrap();
        ctx.store
            .set_last_push(&t, now - std::time::Duration::from_secs(7200))
            .unwrap();
        let report = listed(&mut ctx);
        assert_eq!(
            report.data["remotes"][0]["last_pull"],
            (now - std::time::Duration::from_secs(240)).to_string()
        );
        assert_eq!(
            report.data["remotes"][0]["last_push"],
            (now - std::time::Duration::from_secs(7200)).to_string()
        );
        assert!(report.human.contains("pulled 4m ago"), "{}", report.human);
        assert!(report.human.contains("pushed 2h ago"), "{}", report.human);
    }

    fn listed(ctx: &mut Context) -> Report {
        run_remote(
            ctx,
            RemoteArgs {
                command: RemoteCommand::List,
            },
        )
        .unwrap()
    }

    #[test]
    fn a_stale_remote_is_pulled_before_a_read_and_a_fresh_one_is_not() {
        let mut ctx = context();
        ctx.config.remotes.push(RemoteConfig {
            name: RemoteName("t".into()),
            helper: "t".into(),
            url: "t::".into(),
            credentials: vec![],
            stale: Some(Duration::from_secs(60)),
            deadline: None,
            path: None,
        });
        ctx.launcher = Box::new(EchoLauncher {
            pulled: RefCell::new(Some(PullOutcome {
                objects: vec![incoming("aaaa", "from upstream")],
                ..PullOutcome::default()
            })),
        });
        maybe_pull_stale(&mut ctx).unwrap();
        assert_eq!(ctx.store.all().unwrap().len(), 1);
        ctx.launcher = Box::new(EchoLauncher {
            pulled: RefCell::new(Some(PullOutcome {
                objects: vec![incoming("bbbb", "second")],
                ..PullOutcome::default()
            })),
        });
        maybe_pull_stale(&mut ctx).unwrap();
        assert_eq!(
            ctx.store.all().unwrap().len(),
            1,
            "a fresh remote is not pulled again"
        );
    }

    #[test]
    fn no_pull_answers_from_the_store_and_never_launches_a_helper() {
        let mut ctx = context();
        ctx.no_pull = true;
        ctx.config.remotes.push(RemoteConfig {
            name: RemoteName("t".into()),
            helper: "t".into(),
            url: "t::".into(),
            credentials: vec![],
            stale: Some(Duration::from_secs(60)),
            deadline: None,
            path: None,
        });
        // A launcher that cannot produce a helper, so reaching for one at all
        // would leave a PullFailed notice behind.
        ctx.launcher = Box::new(FailingLauncher);
        maybe_pull_stale(&mut ctx).unwrap();
        assert!(ctx.store.all().unwrap().is_empty());
        assert!(ctx.store.notices().unwrap().is_empty());
        assert!(
            ctx.store
                .last_pull(&RemoteName("t".into()))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn a_remote_without_stale_is_never_auto_pulled() {
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
        ctx.launcher = Box::new(EchoLauncher {
            pulled: RefCell::new(Some(PullOutcome {
                objects: vec![incoming("aaaa", "x")],
                ..PullOutcome::default()
            })),
        });
        maybe_pull_stale(&mut ctx).unwrap();
        assert!(ctx.store.all().unwrap().is_empty());
    }

    #[test]
    fn an_unreachable_stale_remote_records_a_notice_and_does_not_advance_last_pull() {
        let mut ctx = context();
        ctx.config.remotes.push(RemoteConfig {
            name: RemoteName("t".into()),
            helper: "t".into(),
            url: "t::".into(),
            credentials: vec![],
            stale: Some(Duration::from_secs(60)),
            deadline: None,
            path: None,
        });
        ctx.launcher = Box::new(FailingLauncher);
        maybe_pull_stale(&mut ctx).unwrap();
        assert!(ctx.store.all().unwrap().is_empty());
        assert!(
            ctx.store
                .last_pull(&RemoteName("t".into()))
                .unwrap()
                .is_none()
        );
        assert!(matches!(
            ctx.store.notices().unwrap().last(),
            Some(dam_application::Notice::PullFailed { remote, .. }) if remote.0 == "t"
        ));
    }
}
