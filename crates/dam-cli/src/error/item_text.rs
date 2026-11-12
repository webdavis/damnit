use std::fmt;

use dam_application::ItemRefusal;

pub(super) struct ItemText<'a>(pub(super) &'a ItemRefusal);

impl fmt::Display for ItemText<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            ItemRefusal::NothingChanged => f.write_str("that leaves the object as it is"),
            ItemRefusal::Declined(hash) => write!(
                f,
                "the owner declined this as {}; it is not offered again",
                hash.short()
            ),
        }
    }
}
