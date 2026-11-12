use dam_domain::ItemHash;

#[derive(Debug, PartialEq, Eq)]
pub enum ItemRefusal {
    NothingChanged,
    Declined(ItemHash),
}

impl ItemRefusal {
    pub fn name(&self) -> &'static str {
        match self {
            ItemRefusal::NothingChanged => "nothing_changed",
            ItemRefusal::Declined(_) => "item_declined",
        }
    }
}
