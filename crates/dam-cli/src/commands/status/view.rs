use dam_application::Unpushed;
use dam_domain::{CommitId, Oid};
use serde::Serialize;

use super::notices::NoticeRow;
use super::rows::{ChangeRow, ConflictRow};
use super::waiting::ItemRow;
use crate::layout::counted;
use crate::view::{Human, Line, View, displays};

#[derive(Serialize)]
pub(super) struct StatusView<'a> {
    pub(super) staged: Vec<ChangeRow<'a>>,
    pub(super) unstaged: Vec<ChangeRow<'a>>,
    pub(super) waiting: Vec<ItemRow<'a>>,
    pub(super) conflicts: Vec<ConflictRow<'a>>,
    pub(super) notices: Vec<NoticeRow<'a>>,
    pub(super) unpushed: Vec<UnpushedRow<'a>>,
    pub(super) warnings: Vec<String>,
    #[serde(skip)]
    pub(super) owner: &'static str,
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
        let mut section = |title: &str, lines: Vec<Line>| {
            if !lines.is_empty() {
                human.push(Line::Row(title.to_string()));
                human.extend(lines);
            }
        };
        let rows = |lines: Vec<String>| -> Vec<Line> {
            lines
                .into_iter()
                .map(|l| Line::Row(format!("  {l}")))
                .collect()
        };
        let changes = |rows: &[ChangeRow]| -> Vec<Line> {
            rows.iter()
                .flat_map(|r| r.own_lines("  ", self.owner))
                .collect()
        };
        section("Staged:", changes(&self.staged));
        let waiting = self.waiting.iter().flat_map(|r| r.lines(self.owner));
        section(
            "Not staged:",
            changes(&self.unstaged).into_iter().chain(waiting).collect(),
        );
        section(
            "Conflicts:",
            self.conflicts.iter().flat_map(ConflictRow::lines).collect(),
        );
        section(
            "Notices:",
            self.notices
                .iter()
                .map(|n| Line::Whole(format!("  {}", n.line())))
                .collect(),
        );
        section(
            "Unpushed:",
            rows(
                self.unpushed
                    .iter()
                    .filter(|u| u.commits > 0)
                    .map(|u| format!("{}: {}", u.remote, counted(u.commits, "commit", "commits")))
                    .collect(),
            ),
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
    #[serde(skip)]
    pub(super) owner: &'static str,
}

impl View for DiffView<'_> {
    fn human(&self) -> Human {
        if self.changes.is_empty() {
            return Human::Lines(vec!["no changes".to_string()]);
        }
        Human::Mixed(
            self.changes
                .iter()
                .flat_map(|c| c.own_lines("", self.owner))
                .collect(),
        )
    }
}
