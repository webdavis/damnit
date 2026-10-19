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
        Human::list(
            self.remotes.iter().map(|r| r.cells(self.now)).collect(),
            "no remotes configured",
        )
    }
}

impl RemoteRow<'_> {
    fn cells(&self, now: Timestamp) -> Vec<String> {
        vec![
            self.name.to_string(),
            self.url.to_string(),
            self.path.map(|p| format!("path {p}")).unwrap_or_default(),
            self.stale_seconds
                .map(|seconds| format!("stale {}", in_the_largest_whole_unit(seconds)))
                .unwrap_or_default(),
            when_phrase("pulled", self.last_pull, now),
            when_phrase("pushed", self.last_push, now),
        ]
    }
}

fn in_the_largest_whole_unit(seconds: u64) -> String {
    match seconds {
        0 => "0s".to_string(),
        s if s % 3600 == 0 => format!("{}h", s / 3600),
        s if s % 60 == 0 => format!("{}m", s / 60),
        s => format!("{s}s"),
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
mod tests;
