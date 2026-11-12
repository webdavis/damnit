use dam_application::{ItemRefusal, Refusal};
use dam_domain::ItemHash;

use super::oid;

pub(super) fn item_refusals_with_their_text() -> Vec<(Refusal, &'static str)> {
    [
        (
            ItemRefusal::NothingChanged,
            "that leaves the object as it is",
        ),
        (
            ItemRefusal::Declined(ItemHash::new(oid(1))),
            "the owner declined this as 0101010; it is not offered again",
        ),
    ]
    .into_iter()
    .map(|(refusal, text)| (Refusal::Item(refusal), text))
    .collect()
}
