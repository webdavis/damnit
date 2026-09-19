use dam_adapters::{append_remote, load_config};
use dam_application::{Notice, RemoteConfig, RemoteName, pull};

use crate::args::{RemoteArgs, RemoteCommand};
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;

pub fn run_remote(ctx: &mut Context, args: RemoteArgs) -> Result<Report, CliError> {
    match args.command {
        RemoteCommand::Add { name, url } => {
            append_remote(&ctx.config_path, &name, &url)?;
            ctx.config = load_config(&ctx.config_path)?;
            Ok(Report {
                human: format!("added {name} ({url})"),
                data: serde_json::json!({ "name": name, "url": url }),
            })
        }
        RemoteCommand::List => Ok(Report {
            human: ctx
                .config
                .remotes
                .iter()
                .map(remote_line)
                .collect::<Vec<_>>()
                .join("\n"),
            data: serde_json::json!({ "remotes": ctx.config.remotes.iter().map(remote_json).collect::<Vec<_>>() }),
        }),
    }
}

fn remote_line(r: &RemoteConfig) -> String {
    let mut line = format!("{}  {}::", r.name.0, r.helper);
    if let Some(p) = &r.path {
        line.push_str(&format!("  path {}", p.as_str()));
    }
    if let Some(s) = r.stale {
        line.push_str(&format!("  stale {}s", s.as_secs()));
    }
    line
}

fn remote_json(r: &RemoteConfig) -> serde_json::Value {
    serde_json::json!({
        "name": r.name.0,
        "helper": r.helper,
        "path": r.path.as_ref().map(|p| p.as_str()),
        "stale_seconds": r.stale.map(|s| s.as_secs()),
    })
}

/// Pulls each remote whose `stale` is set and whose last pull is older than
/// it. A failure to reach a remote is a notice, not an error, so a read
/// never fails because the network did.
pub fn maybe_pull_stale(ctx: &mut Context) -> Result<(), CliError> {
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
            ctx.store.as_ref(),
            ctx.launcher.as_ref(),
            ctx.credentials.as_ref(),
            ctx.clock.as_ref(),
            ctx.random.as_mut(),
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
    use dam_application::{RemoteConfig, RemoteName};
    use dam_protocol::{PullResponse, WireObject, WireTask};
    use std::cell::RefCell;
    use std::time::Duration;

    fn wire(oid: &str, subject: &str) -> WireObject {
        WireObject {
            oid: oid.into(),
            remote_id: Some(format!("r-{oid}")),
            kind: "task".into(),
            subject: subject.into(),
            body: String::new(),
            path: String::new(),
            labels: vec![],
            depends: vec![],
            reminders: vec![],
            recurrence: None,
            task: Some(WireTask {
                done: false,
                priority: 4,
                due: None,
                deadline: None,
                event: None,
            }),
            event: None,
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
                    url: "todoist::".into(),
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
        assert!(report.human.contains("todoist"));
        assert_eq!(ctx.config.remote("todoist").unwrap().helper, "todoist");
    }

    #[test]
    fn a_stale_remote_is_pulled_before_a_read_and_a_fresh_one_is_not() {
        let mut ctx = context();
        ctx.config.remotes.push(RemoteConfig {
            name: RemoteName("t".into()),
            helper: "t".into(),
            credentials: vec![],
            stale: Some(Duration::from_secs(60)),
            path: None,
        });
        ctx.launcher = Box::new(EchoLauncher {
            pulled: RefCell::new(Some(PullResponse {
                objects: vec![wire("aaaa", "from upstream")],
                removed: vec![],
                sync: None,
            })),
        });
        maybe_pull_stale(&mut ctx).unwrap();
        assert_eq!(ctx.store.all().unwrap().len(), 1);
        ctx.launcher = Box::new(EchoLauncher {
            pulled: RefCell::new(Some(PullResponse {
                objects: vec![wire("bbbb", "second")],
                removed: vec![],
                sync: None,
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
    fn a_remote_without_stale_is_never_auto_pulled() {
        let mut ctx = context();
        ctx.config.remotes.push(RemoteConfig {
            name: RemoteName("t".into()),
            helper: "t".into(),
            credentials: vec![],
            stale: None,
            path: None,
        });
        ctx.launcher = Box::new(EchoLauncher {
            pulled: RefCell::new(Some(PullResponse {
                objects: vec![wire("aaaa", "x")],
                removed: vec![],
                sync: None,
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
            credentials: vec![],
            stale: Some(Duration::from_secs(60)),
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
