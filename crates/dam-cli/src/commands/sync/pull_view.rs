use dam_application::PullReport;
use serde::Serialize;

use super::pulled_hook_view::PulledHookRow;
use crate::layout::counted;
use crate::style::{Mark, Paint, advice, heading, row};
use crate::view::{Human, View};

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
    creates_matched: usize,
    values_kept: usize,
    deletes_confirmed: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    task_pulled: Option<PulledHookRow>,
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
            creates_matched: r.creates_matched,
            values_kept: r.values_kept,
            deletes_confirmed: r.deletes_confirmed,
            task_pulled: r.task_pulled.as_ref().map(PulledHookRow::new),
        }
    }

    pub(super) fn lines(&self, paint: Paint, name: &str) -> Vec<String> {
        let mut lines = vec![row(paint, self.mark(), 2, &self.line(name))];
        if self.conflicts > 0 {
            lines.push(advice(
                paint,
                4,
                "see dam status, settle each with dam resolve",
            ));
        }
        if self.removed_upstream > 0 {
            lines.push(advice(paint, 4, "see dam status, drop each with dam rm"));
        }
        let taken_from_the_record = [
            (self.creates_matched, "create matched", "creates matched"),
            (self.values_kept, "value kept", "values kept"),
            (
                self.deletes_confirmed,
                "delete confirmed",
                "deletes confirmed",
            ),
        ];
        for (n, one, many) in taken_from_the_record.into_iter().filter(|(n, ..)| *n > 0) {
            lines.push(row(paint, Mark::Quiet, 4, &counted(n, one, many)));
        }
        if let Some(run) = &self.task_pulled {
            lines.push(row(paint, run.mark(), 4, &run.line()));
        }
        lines
    }

    fn mark(&self) -> Mark {
        match self.conflicts + self.removed_upstream {
            0 => Mark::Good,
            _ => Mark::Warn,
        }
    }

    fn line(&self, name: &str) -> String {
        let counts = [
            (self.created, "new", "new"),
            (self.updated, "updated", "updated"),
            (self.unchanged, "unchanged", "unchanged"),
            (self.conflicts, "conflict", "conflicts"),
            (
                self.removed_upstream,
                "removed upstream",
                "removed upstream",
            ),
        ];
        let mut shown: Vec<String> = counts
            .into_iter()
            .filter(|(n, ..)| *n > 0)
            .map(|(n, one, many)| counted(n, one, many))
            .collect();
        if self.created + self.updated + self.conflicts + self.removed_upstream == 0 {
            shown.insert(0, "nothing new".to_string());
        }
        format!("{name}: {}", shown.join(", "))
    }
}

impl View for Pulled<'_> {
    fn human(&self) -> Human {
        Human::styled(|paint| {
            let mut lines = vec![heading(paint, "pull", "what each remote sent")];
            match self.remotes.as_slice() {
                [] => lines.push(row(paint, Mark::Quiet, 2, "no remotes to pull from")),
                remotes => lines.extend(remotes.iter().flat_map(|r| r.lines(paint, r.remote))),
            }
            lines
        })
    }
}
