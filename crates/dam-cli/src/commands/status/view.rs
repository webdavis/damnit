use dam_application::Unpushed;
use dam_domain::{CommitId, Oid};
use serde::Serialize;

use super::notices::NoticeRow;
use super::rows::{ChangeRow, ConflictRow};
use crate::layout::counted;
use crate::view::{Human, Line, View, displays};

#[derive(Serialize)]
pub(super) struct StatusView<'a> {
    pub(super) staged: Vec<ChangeRow<'a>>,
    pub(super) unstaged: Vec<ChangeRow<'a>>,
    pub(super) conflicts: Vec<ConflictRow<'a>>,
    pub(super) notices: Vec<NoticeRow<'a>>,
    pub(super) unpushed: Vec<UnpushedRow<'a>>,
    pub(super) warnings: Vec<String>,
}

#[derive(Serialize)]
pub(super) struct UnpushedRow<'a> {
    remote: &'a str,
    commits: usize,
    #[serde(serialize_with = "displays")]
    commit_ids: &'a [CommitId],
    #[serde(serialize_with = "displays")]
    oids: &'a [Oid],
}

impl<'a> UnpushedRow<'a> {
    pub(super) fn new(u: &'a Unpushed) -> UnpushedRow<'a> {
        UnpushedRow {
            remote: &u.remote.0,
            commits: u.commit_ids.len(),
            commit_ids: &u.commit_ids,
            oids: &u.oids,
        }
    }
}

impl View for StatusView<'_> {
    fn human(&self) -> Human {
        let mut human: Vec<Line> = self.warnings.iter().cloned().map(Line::Whole).collect();
        let mut section = |title: &str, lines: Vec<String>, line: fn(String) -> Line| {
            if !lines.is_empty() {
                human.push(Line::Row(title.to_string()));
                human.extend(lines.into_iter().map(|l| line(format!("  {l}"))));
            }
        };
        section(
            "Staged:",
            self.staged.iter().map(ChangeRow::line).collect(),
            Line::Row,
        );
        section(
            "Not staged:",
            self.unstaged.iter().map(ChangeRow::line).collect(),
            Line::Row,
        );
        section(
            "Conflicts:",
            self.conflicts.iter().map(ConflictRow::line).collect(),
            Line::Row,
        );
        section(
            "Notices:",
            self.notices.iter().map(NoticeRow::line).collect(),
            Line::Whole,
        );
        section(
            "Unpushed:",
            self.unpushed
                .iter()
                .filter(|u| u.commits > 0)
                .map(|u| format!("{}: {}", u.remote, counted(u.commits, "commit", "commits")))
                .collect(),
            Line::Row,
        );
        if human.len() == self.warnings.len() {
            human.push(Line::Row("nothing staged, nothing changed".to_string()));
        }
        Human::Mixed(human)
    }
}

#[derive(Serialize)]
pub(super) struct DiffView<'a> {
    pub(super) changes: Vec<ChangeRow<'a>>,
}

impl View for DiffView<'_> {
    fn human(&self) -> Human {
        Human::list(
            self.changes.iter().map(|c| vec![c.line()]).collect(),
            "no changes",
        )
    }
}
