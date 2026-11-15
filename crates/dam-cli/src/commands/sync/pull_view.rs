use dam_application::PullReport;
use serde::Serialize;

use crate::layout::counted;
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
        }
    }

    fn line(&self) -> String {
        let mut line = format!(
            "{}: {} new, {} updated, {} unchanged, {}, {} removed upstream",
            self.remote,
            self.created,
            self.updated,
            self.unchanged,
            counted(self.conflicts, "conflict", "conflicts"),
            self.removed_upstream
        );
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
            line.push_str(&format!("\n  {}", counted(n, one, many)));
        }
        line
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
