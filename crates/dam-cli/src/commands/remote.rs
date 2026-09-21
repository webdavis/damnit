mod listing;

use dam_adapters::{append_remote, load_config};
use dam_application::{Notice, RemoteName, Repositories, pull};

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
        RemoteCommand::List => listing::run(ctx),
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
