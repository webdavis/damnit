use dam_domain::Timestamp;
use serde::Serialize;

use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{Human, View, optional_display, report};

pub(super) fn run(ctx: &Context) -> Result<Report, CliError> {
    let remotes = ctx
        .config
        .remotes
        .iter()
        .map(|r| -> Result<RemoteRow<'_>, CliError> {
            Ok(RemoteRow {
                name: &r.name.0,
                helper: &r.helper,
                url: &r.url,
                path: r.path.as_ref().map(|p| p.as_str()),
                stale_seconds: r.stale.map(|s| s.as_secs()),
                last_pull: ctx.store.last_pull(&r.name)?,
                last_push: ctx.store.last_push(&r.name)?,
            })
        })
        .collect::<Result<_, _>>()?;
    report(&RemoteList {
        remotes,
        now: ctx.clock.now(),
    })
}

#[derive(Serialize)]
struct RemoteList<'a> {
    remotes: Vec<RemoteRow<'a>>,
    #[serde(skip)]
    now: Timestamp,
}

#[derive(Serialize)]
struct RemoteRow<'a> {
    name: &'a str,
    helper: &'a str,
    url: &'a str,
    path: Option<&'a str>,
    stale_seconds: Option<u64>,
    #[serde(serialize_with = "optional_display")]
    last_pull: Option<Timestamp>,
    #[serde(serialize_with = "optional_display")]
    last_push: Option<Timestamp>,
}

impl View for RemoteList<'_> {
    fn human(&self) -> Human {
        Human::Table(self.remotes.iter().map(|r| r.cells(self.now)).collect())
    }
}

impl RemoteRow<'_> {
    fn cells(&self, now: Timestamp) -> Vec<String> {
        let mut cells = vec![self.name.to_string(), self.url.to_string()];
        if let Some(p) = self.path {
            cells.push(format!("path {p}"));
        }
        if let Some(s) = self.stale_seconds {
            cells.push(format!("stale {s}s"));
        }
        cells.push(when_phrase("pulled", self.last_pull, now));
        cells.push(when_phrase("pushed", self.last_push, now));
        cells
    }
}

fn when_phrase(verb: &str, at: Option<Timestamp>, now: Timestamp) -> String {
    match at {
        None => format!("never {verb}"),
        Some(at) => format!(
            "{verb} {}",
            ago_to_one_unit_and_never_in_the_future(now, at)
        ),
    }
}

fn ago_to_one_unit_and_never_in_the_future(now: Timestamp, then: Timestamp) -> String {
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
    use dam_application::RemoteConfig;
    use dam_application::RemoteName;

    #[test]
    fn remote_list_reports_the_last_pull_and_push_as_rfc_3339_a_phrase_or_null() {
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
    fn a_clock_gone_backwards_reads_as_just_now_rather_than_the_future() {
        let now = context().clock.now();
        let later = now + std::time::Duration::from_secs(7200);
        assert_eq!(
            ago_to_one_unit_and_never_in_the_future(now, later),
            "just now"
        );
    }
}
