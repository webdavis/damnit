use std::path::PathBuf;

use dam_domain::{CommitRecord, Object, Oid};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum History {
    Recorded(Vec<CommitRecord>),
    Withheld,
    Unread,
}

pub trait Histories {
    fn of(&self, oid: &Oid) -> History;
}

#[derive(Debug, PartialEq, Eq)]
pub struct FilterFailure {
    pub file: PathBuf,
    pub line: Option<u32>,
    pub fault: FilterFault,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FilterFault {
    Raised { message: String },
    RanOver,
    Returned { returned: String },
}

pub trait FilterFunctions {
    fn keeps(
        &self,
        filter: &str,
        objects: &[Object],
        histories: &dyn Histories,
    ) -> Result<Vec<bool>, FilterFailure>;
}
