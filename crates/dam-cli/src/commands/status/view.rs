use dam_application::Unpushed;
use dam_domain::{CommitId, Oid};
use serde::Serialize;

use super::notices::NoticeRow;
use super::rows::{ChangeRow, ConflictRow};
use crate::view::{Human, View, displays};

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
        let mut human = self.warnings.clone();
        let mut section = |title: &str, lines: Vec<String>| {
            if !lines.is_empty() {
                human.push(title.to_string());
                human.extend(lines.into_iter().map(|l| format!("  {l}")));
            }
        };
        section("Staged:", self.staged.iter().map(ChangeRow::line).collect());
        section(
            "Not staged:",
            self.unstaged.iter().map(ChangeRow::line).collect(),
        );
        section(
            "Conflicts:",
            self.conflicts.iter().map(ConflictRow::line).collect(),
        );
        section(
            "Notices:",
            self.notices.iter().map(NoticeRow::line).collect(),
        );
        section(
            "Unpushed:",
            self.unpushed
                .iter()
                .filter(|u| u.commits > 0)
                .map(|u| format!("{}: {} commit(s)", u.remote, u.commits))
                .collect(),
        );
        if human.len() == self.warnings.len() {
            human.push("nothing staged, nothing changed".to_string());
        }
        Human::Lines(human)
    }
}

#[derive(Serialize)]
pub(super) struct DiffView<'a> {
    pub(super) changes: Vec<ChangeRow<'a>>,
}

impl View for DiffView<'_> {
    fn human(&self) -> Human {
        Human::Lines(self.changes.iter().map(ChangeRow::line).collect())
    }
}
