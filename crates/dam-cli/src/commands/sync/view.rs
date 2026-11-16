use dam_application::PushReport;
use dam_domain::Oid;
use serde::Serialize;

use crate::layout::counted;
use crate::style::{Mark, Paint, advice, row};
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
    pub(super) fn new(r: &'a PushReport) -> PushRow<'a> {
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

    pub(super) fn lines(&self, paint: Paint, name: &str) -> Vec<String> {
        let held = self.held.unwrap_or(0);
        let counts = [
            (self.sent, "sent"),
            (self.failed.len(), "failed"),
            (self.skipped, "skipped"),
            (held, "held"),
        ];
        let shown: Vec<String> = counts
            .into_iter()
            .filter(|(n, _)| *n > 0)
            .map(|(n, word)| counted(n, word, word))
            .collect();
        let text = match shown.is_empty() {
            true => "nothing to send".to_string(),
            false => shown.join(", "),
        };
        let mark = match (self.failed.is_empty(), held) {
            (false, _) => Mark::Bad,
            (true, 0) => Mark::Good,
            (true, _) => Mark::Warn,
        };
        let mut lines = vec![row(paint, mark, 2, &format!("{name}: {text}"))];
        if held > 0 {
            lines.push(advice(
                paint,
                4,
                "see dam status, settle each with dam resolve",
            ));
        }
        lines.extend(self.failed.iter().map(|f| {
            row(
                paint,
                Mark::Bad,
                4,
                &format!("{}: {}", f.oid.short(), f.why),
            )
        }));
        lines
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
        let held = match self.held {
            Some(0) => return line,
            Some(n) => format!("{n} held"),
            None => "some held".to_string(),
        };
        line.push_str(&format!(
            "\n  {held} for a conflict; see dam status, settle with dam resolve"
        ));
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
