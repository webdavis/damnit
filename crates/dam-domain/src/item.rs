mod clash;
mod hash;
mod proposal;

use crate::vocabulary::vocabulary;
use crate::{Change, Details, Oid, Op, Timestamp, TokenAuthor};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemKind {
    New,
    Change,
    Removal,
}

vocabulary!(ItemKind {
    New => "new",
    Change => "change",
    Removal => "removal",
});

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Waiting,
    Added(Timestamp),
    Declined(Timestamp),
    Withdrawn(Timestamp),
}

impl Outcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Outcome::Waiting => "waiting",
            Outcome::Added(_) => "added",
            Outcome::Declined(_) => "declined",
            Outcome::Withdrawn(_) => "withdrawn",
        }
    }

    pub fn at(&self) -> Option<Timestamp> {
        match self {
            Outcome::Waiting => None,
            Outcome::Added(at) | Outcome::Declined(at) | Outcome::Withdrawn(at) => Some(*at),
        }
    }

    pub fn from_parts(word: &str, at: Option<Timestamp>) -> Option<Outcome> {
        match (word, at) {
            ("waiting", None) => Some(Outcome::Waiting),
            ("added", Some(at)) => Some(Outcome::Added(at)),
            ("declined", Some(at)) => Some(Outcome::Declined(at)),
            ("withdrawn", Some(at)) => Some(Outcome::Withdrawn(at)),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub hash: ItemHash,
    pub author: TokenAuthor,
    pub change: Change,
    pub created: Timestamp,
    pub outcome: Outcome,
    pub details: Option<Details>,
    pub set: Option<ItemHash>,
}

impl Item {
    pub fn propose(
        author: TokenAuthor,
        change: Change,
        created: Timestamp,
        fill_with_entropy: &mut dyn FnMut(&mut [u8]),
    ) -> Item {
        let hash = match &change {
            Change::Create { after } => after.oid().clone(),
            Change::Update { .. } | Change::Delete { .. } => Oid::generate(fill_with_entropy),
        };
        Item {
            hash: ItemHash::new(hash),
            author,
            change,
            created,
            outcome: Outcome::Waiting,
            details: None,
            set: None,
        }
    }

    pub fn with_details(self, details: Option<&Details>) -> Item {
        Item {
            details: details.cloned(),
            ..self
        }
    }

    pub fn in_set(self, set: ItemHash) -> Item {
        Item {
            set: Some(set),
            ..self
        }
    }

    pub fn kind(&self) -> ItemKind {
        match self.change.op() {
            Op::Create => ItemKind::New,
            Op::Update => ItemKind::Change,
            Op::Delete => ItemKind::Removal,
        }
    }

    pub fn target(&self) -> &Oid {
        self.change.oid()
    }

    pub fn is_waiting(&self) -> bool {
        self.outcome == Outcome::Waiting
    }
}

pub use clash::{clashing_fields, fields_the_owner_moved};
pub use hash::ItemHash;
pub use proposal::same_proposal;

#[cfg(test)]
mod clash_tests;
#[cfg(test)]
mod proposal_tests;
#[cfg(test)]
mod tests;
