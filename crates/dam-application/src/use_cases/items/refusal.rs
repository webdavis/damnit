use dam_domain::{Field, ItemHash, Oid};

#[derive(Debug, PartialEq, Eq)]
pub enum ItemRefusal {
    NothingChanged,
    Declined(ItemHash),
    NotAnObject(ItemHash),
    NotYours(ItemHash),
    NotWaiting {
        hash: ItemHash,
        outcome: &'static str,
    },
    TargetGone {
        hash: ItemHash,
        target: Oid,
    },
    TargetNotCommitted {
        hash: ItemHash,
        target: Oid,
    },
    TargetStaged {
        hash: ItemHash,
        target: Oid,
    },
    Clash {
        hash: ItemHash,
        target: Oid,
        fields: Vec<Field>,
    },
    EditedSinceAdded {
        hash: ItemHash,
        target: Oid,
    },
    SubjectNamesNoFolder {
        hash: ItemHash,
        target: Oid,
    },
}

impl ItemRefusal {
    pub fn name(&self) -> &'static str {
        match self {
            ItemRefusal::NothingChanged => "nothing_changed",
            ItemRefusal::Declined(_) => "item_declined",
            ItemRefusal::NotAnObject(_) => "item_not_an_object",
            ItemRefusal::NotYours(_) => "not_your_item",
            ItemRefusal::NotWaiting { .. } => "item_not_waiting",
            ItemRefusal::TargetGone { .. } => "item_target_gone",
            ItemRefusal::TargetNotCommitted { .. } => "target_not_committed",
            ItemRefusal::TargetStaged { .. } => "target_staged",
            ItemRefusal::Clash { .. } => "item_clash",
            ItemRefusal::EditedSinceAdded { .. } => "item_edited_since_added",
            ItemRefusal::SubjectNamesNoFolder { .. } => "item_subject_names_no_folder",
        }
    }
}
