use dam_application::PushReport;
use dam_domain::Oid;
use serde::Serialize;

use crate::view::{Human, View, display};

#[derive(Serialize)]
pub(super) struct Pushed<'a> {
    pub(super) remotes: Vec<PushRow<'a>>,
}

#[derive(Serialize)]
pub(crate) struct PushRow<'a> {
    remote: &'a str,
    sent: usize,
    succeeded: usize,
    skipped: usize,
    held: Option<usize>,
    failed: Vec<FailedPush<'a>>,
}

#[derive(Serialize)]
struct FailedPush<'a> {
    #[serde(serialize_with = "display")]
    oid: &'a Oid,
    why: &'a str,
}

impl<'a> PushRow<'a> {
    pub(crate) fn new(r: &'a PushReport) -> PushRow<'a> {
        PushRow {
            remote: &r.remote.0,
            sent: r.sent,
            succeeded: r.succeeded,
            skipped: r.skipped,
            held: r.held,
            failed: r
                .failed
                .iter()
                .map(|(oid, why)| FailedPush { oid, why })
                .collect(),
        }
    }

    pub(crate) fn counts(&self) -> String {
        format!(
            "{} sent, {} ok, {} failed, {} skipped",
            self.sent,
            self.succeeded,
            self.failed.len(),
            self.skipped
        )
    }

    fn line(&self) -> String {
        let mut line = format!("{}: {}", self.remote, self.counts());
        for f in &self.failed {
            line.push_str(&format!("\n  {}: {}", f.oid.short(), f.why));
        }
        if let Some(held) = self.held_for_a_conflict() {
            line.push_str(&format!("\n  {held}"));
        }
        line
    }

    pub(crate) fn held_for_a_conflict(&self) -> Option<String> {
        let held = match self.held {
            Some(0) => return None,
            Some(n) => format!("{n} held"),
            None => "some held".to_string(),
        };
        Some(format!(
            "{held} for a conflict; see dam status, settle with dam resolve"
        ))
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
