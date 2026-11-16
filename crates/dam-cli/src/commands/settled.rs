use dam_domain::{Item, ItemHash, Oid, Timestamp};
use serde::Serialize;

use crate::view::{Human, View, display, optional_display};

#[derive(Serialize)]
pub(crate) struct Settled<'a> {
    #[serde(serialize_with = "display")]
    hash: &'a ItemHash,
    outcome: &'static str,
    #[serde(serialize_with = "optional_display")]
    at: Option<Timestamp>,
    #[serde(serialize_with = "display")]
    oid: &'a Oid,
    item: &'static str,
    subject: &'a str,
    from: &'a str,
}

impl<'a> Settled<'a> {
    pub(crate) fn new(item: &'a Item) -> Settled<'a> {
        Settled {
            hash: &item.hash,
            outcome: item.outcome.as_str(),
            at: item.outcome.at(),
            oid: item.target(),
            item: item.kind().as_str(),
            subject: &item.change.object().base().subject,
            from: &item.author.name,
        }
    }
}

impl View for Settled<'_> {
    fn human(&self) -> Human {
        let mut cells = vec![self.outcome, self.hash.short()];
        if self.oid != self.hash.oid() {
            cells.push(self.oid.short());
        }
        cells.extend([self.subject, self.from]);
        Human::Lines(vec![cells.join("  ")])
    }
}

#[derive(Serialize)]
pub(crate) struct SettledSet<'a> {
    #[serde(serialize_with = "display")]
    set: &'a ItemHash,
    items: Vec<Settled<'a>>,
}

impl<'a> SettledSet<'a> {
    pub(crate) fn new(set: &'a ItemHash, items: &'a [Item]) -> SettledSet<'a> {
        SettledSet {
            set,
            items: items.iter().map(Settled::new).collect(),
        }
    }
}

impl View for SettledSet<'_> {
    fn human(&self) -> Human {
        Human::Lines(
            self.items
                .iter()
                .flat_map(|item| match item.human() {
                    Human::Lines(lines) => lines,
                    _ => Vec::new(),
                })
                .collect(),
        )
    }
}
