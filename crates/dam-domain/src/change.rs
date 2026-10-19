use std::fmt;

use crate::vocabulary::vocabulary;
use crate::{Object, Oid, Timestamp};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Create,
    Update,
    Delete,
}

vocabulary!(Op {
    Create => "create",
    Update => "update",
    Delete => "delete",
});

#[allow(
    clippy::large_enum_variant,
    reason = "every variant holds its sides by value, with no indirection"
)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
    Create { after: Object },
    Update { before: Object, after: Object },
    Delete { before: Object },
}

impl Change {
    pub fn between(before: Option<Object>, after: Option<Object>) -> Option<Change> {
        match (before, after) {
            (None, None) => None,
            (None, Some(after)) => Some(Change::Create { after }),
            (Some(before), None) => Some(Change::Delete { before }),
            (Some(before), Some(after)) if before == after => None,
            (Some(before), Some(after)) => Some(Change::Update { before, after }),
        }
    }

    pub fn op(&self) -> Op {
        match self {
            Change::Create { .. } => Op::Create,
            Change::Update { .. } => Op::Update,
            Change::Delete { .. } => Op::Delete,
        }
    }

    pub fn before(&self) -> Option<&Object> {
        match self {
            Change::Create { .. } => None,
            Change::Update { before, .. } | Change::Delete { before } => Some(before),
        }
    }

    pub fn after(&self) -> Option<&Object> {
        match self {
            Change::Create { after } | Change::Update { after, .. } => Some(after),
            Change::Delete { .. } => None,
        }
    }

    pub fn object(&self) -> &Object {
        match self {
            Change::Create { after } | Change::Update { after, .. } => after,
            Change::Delete { before } => before,
        }
    }

    pub fn oid(&self) -> &Oid {
        self.object().oid()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CommitId(Oid);

impl CommitId {
    pub fn generate(fill: &mut dyn FnMut(&mut [u8])) -> CommitId {
        CommitId(Oid::generate(fill))
    }

    pub fn parse(text: &str) -> Result<CommitId, crate::OidError> {
        Oid::parse(text).map(CommitId)
    }

    pub fn short(&self) -> &str {
        self.0.short()
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for CommitId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommitRecord {
    pub id: CommitId,
    pub message: String,
    pub at: Timestamp,
    pub changes: Vec<Change>,
}

mod fields;

pub use fields::{changed_fields, kind_change, merge_fields, touched_fields};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod touched;

#[cfg(test)]
mod differing;

#[cfg(test)]
mod merging;
