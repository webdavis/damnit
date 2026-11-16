use std::fmt;

use crate::Oid;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ItemHash(Oid);

impl ItemHash {
    pub fn new(oid: Oid) -> ItemHash {
        ItemHash(oid)
    }

    pub fn oid(&self) -> &Oid {
        &self.0
    }

    pub fn short(&self) -> &str {
        self.0.short()
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for ItemHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
