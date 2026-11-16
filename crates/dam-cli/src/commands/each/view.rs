use dam_application::{SetChange, SetChanges};
use dam_domain::{ItemHash, Oid};
use serde::Serialize;

use crate::commands::status::ChangeRow;
use crate::view::{Human, Line, View, display, displays};

#[derive(Serialize)]
pub(crate) struct SetView<'a> {
    preview: bool,
    count: usize,
    #[serde(serialize_with = "displays")]
    oids: &'a [Oid],
    changes: Vec<ChangeView<'a>>,
    #[serde(skip)]
    owner: &'a str,
}

#[derive(Serialize)]
struct ChangeView<'a> {
    #[serde(flatten)]
    row: ChangeRow<'a>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "optional_hash"
    )]
    item: Option<&'a ItemHash>,
    #[serde(skip_serializing_if = "Option::is_none")]
    answer: Option<&'static str>,
}

fn optional_hash<S: serde::Serializer>(
    hash: &Option<&ItemHash>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match hash {
        Some(hash) => display(hash, serializer),
        None => serializer.serialize_none(),
    }
}

impl<'a> SetView<'a> {
    pub(crate) fn of(changes: &'a SetChanges, owner: &'a str) -> SetView<'a> {
        SetView {
            preview: changes.preview,
            count: changes.oids.len(),
            oids: &changes.oids,
            changes: changes
                .changes
                .iter()
                .map(|c| ChangeView::of(c, owner))
                .collect(),
            owner,
        }
    }
}

impl<'a> ChangeView<'a> {
    fn of(change: &'a SetChange, owner: &'a str) -> ChangeView<'a> {
        let sent = change.sent.as_ref();
        ChangeView {
            row: ChangeRow::new(&change.change, sent.map(|s| &s.item.author), false)
                .sent(owner, None),
            item: sent.map(|s| &s.item.hash),
            answer: sent.map(|s| s.answer.as_str()),
        }
    }
}

impl SetView<'_> {
    pub(crate) fn is_preview(&self) -> bool {
        self.preview
    }
}

impl View for SetView<'_> {
    fn human(&self) -> Human {
        if self.count == 0 {
            return Human::Lines(vec!["nothing matches".into()]);
        }
        let mut lines: Vec<Line> = self
            .changes
            .iter()
            .flat_map(|c| c.row.own_lines("", self.owner))
            .collect();
        let changed = self.changes.len();
        lines.push(Line::Whole(match self.preview {
            true => format!(
                "preview: {changed} of {} would change, nothing written",
                self.count
            ),
            false => format!("changed {changed} of {}", self.count),
        }));
        Human::Mixed(lines)
    }
}
