use dam_domain::{Blockers, Field, ItemHash, Oid};

#[derive(Debug, PartialEq, Eq)]
pub enum ItemRefusal {
    NothingChanged,
    Declined(ItemHash),
    NotAnObject(ItemHash),
    AlreadyCommitted(ItemHash),
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
    RemovesAChangedTask {
        hash: ItemHash,
        target: Oid,
        fields: Vec<Field>,
    },
    Cycle {
        hash: ItemHash,
        target: Oid,
        path: Vec<Oid>,
    },
    Blocked {
        hash: ItemHash,
        target: Oid,
        blockers: Blockers,
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
            ItemRefusal::AlreadyCommitted(_) => "item_already_committed",
            ItemRefusal::NotYours(_) => "not_your_item",
            ItemRefusal::NotWaiting { .. } => "item_not_waiting",
            ItemRefusal::TargetGone { .. } => "item_target_gone",
            ItemRefusal::TargetNotCommitted { .. } => "target_not_committed",
            ItemRefusal::TargetStaged { .. } => "target_staged",
            ItemRefusal::Clash { .. } => "item_clash",
            ItemRefusal::RemovesAChangedTask { .. } => "item_removes_a_changed_task",
            ItemRefusal::Cycle { .. } => "item_cycle",
            ItemRefusal::Blocked { .. } => "item_blocked",
            ItemRefusal::EditedSinceAdded { .. } => "item_edited_since_added",
            ItemRefusal::SubjectNamesNoFolder { .. } => "item_subject_names_no_folder",
        }
    }
}
