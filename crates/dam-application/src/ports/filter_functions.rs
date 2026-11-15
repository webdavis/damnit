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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionFailure {
    pub file: PathBuf,
    pub line: Option<u32>,
    pub fault: FunctionFault,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FunctionFault {
    Raised { message: String },
    RanOver,
    Returned { returned: String },
    Requested { why: String },
}

pub trait FilterFunctions {
    fn keeps(
        &self,
        filter: &str,
        objects: &[Object],
        histories: &dyn Histories,
    ) -> Result<Vec<bool>, FunctionFailure>;
}
