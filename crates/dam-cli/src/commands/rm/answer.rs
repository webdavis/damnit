use dam_application::ItemAndTask;
use dam_domain::{ItemHash, Oid};
use serde::Serialize;

use crate::view::{Human, View, optional_display};

#[derive(Serialize)]
pub(super) struct Answer<'a> {
    answer: &'static str,
    #[serde(serialize_with = "optional_display")]
    item: Option<&'a ItemHash>,
    #[serde(serialize_with = "optional_display")]
    oid: Option<&'a Oid>,
}

impl<'a> Answer<'a> {
    pub(super) fn about(answer: &'static str, about: &'a ItemAndTask) -> Answer<'a> {
        Answer {
            answer,
            item: Some(&about.item),
            oid: Some(&about.oid),
        }
    }
}

impl View for Answer<'_> {
    fn human(&self) -> Human {
        let mut cells = vec![self.answer];
        if let Some(item) = self.item {
            cells.push(item.short());
        }
        if let Some(oid) = self
            .oid
            .filter(|oid| Some(*oid) != self.item.map(ItemHash::oid))
        {
            cells.push(oid.short());
        }
        Human::Lines(vec![cells.join("  ")])
    }
}
