use dam_application::{PullReport, PushReport};
use dam_domain::Oid;
use serde::Serialize;

use crate::layout::counted;
use crate::view::{Human, View, display};

#[derive(Serialize)]
pub(super) struct Pushed<'a> {
    pub(super) remotes: Vec<PushRow<'a>>,
}

#[derive(Serialize)]
pub(super) struct PushRow<'a> {
    remote: &'a str,
    sent: usize,
    succeeded: usize,
    skipped: usize,
    failed: Vec<FailedPush<'a>>,
}

#[derive(Serialize)]
struct FailedPush<'a> {
    #[serde(serialize_with = "display")]
    oid: &'a Oid,
    why: &'a str,
}

impl<'a> PushRow<'a> {
    pub(super) fn new(r: &'a PushReport) -> PushRow<'a> {
        PushRow {
            remote: &r.remote.0,
            sent: r.sent,
            succeeded: r.succeeded,
            skipped: r.skipped,
            failed: r
                .failed
                .iter()
                .map(|(oid, why)| FailedPush { oid, why })
                .collect(),
        }
    }

    fn line(&self) -> String {
        let mut line = format!(
            "{}: {} sent, {} ok, {} failed, {} skipped",
            self.remote,
            self.sent,
            self.succeeded,
            self.failed.len(),
            self.skipped
        );
        for f in &self.failed {
            line.push_str(&format!("\n  {}: {}", f.oid.short(), f.why));
        }
        line
    }
}

impl View for Pushed<'_> {
    fn human(&self) -> Human {
        match self.remotes.as_slice() {
            [] => Human::Lines(vec!["no remotes to push to".to_string()]),
            remotes => Human::Lines(remotes.iter().map(PushRow::line).collect()),
        }
    }
}

#[derive(Serialize)]
pub(super) struct Pulled<'a> {
    pub(super) remotes: Vec<PullRow<'a>>,
}

#[derive(Serialize)]
pub(super) struct PullRow<'a> {
    remote: &'a str,
    created: usize,
    updated: usize,
    unchanged: usize,
    conflicts: usize,
    removed_upstream: usize,
}

impl<'a> PullRow<'a> {
    pub(super) fn new(r: &'a PullReport) -> PullRow<'a> {
        PullRow {
            remote: &r.remote.0,
            created: r.created,
            updated: r.updated,
            unchanged: r.unchanged,
            conflicts: r.conflicts,
            removed_upstream: r.removed_upstream,
        }
    }

    fn line(&self) -> String {
        format!(
            "{}: {} new, {} updated, {} unchanged, {}, {} removed upstream",
            self.remote,
            self.created,
            self.updated,
            self.unchanged,
            counted(self.conflicts, "conflict", "conflicts"),
            self.removed_upstream
        )
    }
}

impl View for Pulled<'_> {
    fn human(&self) -> Human {
        match self.remotes.as_slice() {
            [] => Human::Lines(vec!["no remotes to pull from".to_string()]),
            remotes => Human::Lines(remotes.iter().map(PullRow::line).collect()),
        }
    }
}

#[derive(Serialize)]
pub(super) struct Resolved<'a> {
    #[serde(serialize_with = "display")]
    pub(super) oid: &'a Oid,
    pub(super) side: &'static str,
}

impl View for Resolved<'_> {
    fn human(&self) -> Human {
        Human::Lines(vec![format!(
            "{} resolved with {}",
            self.oid.short(),
            self.side
        )])
    }
}
