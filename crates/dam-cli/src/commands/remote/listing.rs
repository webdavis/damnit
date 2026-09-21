//! `dam remote list`: every remote with how fresh it is.

use dam_application::RemoteConfig;
use dam_domain::Timestamp;

use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;

/// Every remote with the times it was last reached, so a client header can
/// say how fresh what it is showing is.
pub(super) fn run(ctx: &Context) -> Result<Report, CliError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::{RemoteArgs, RemoteCommand};
    use crate::commands::remote::run_remote;
    use crate::testing::context;
    use dam_application::RemoteName;

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
}
