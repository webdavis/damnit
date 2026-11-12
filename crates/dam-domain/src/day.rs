use crate::{Date, Oid, Timestamp};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Day {
    pub day: Date,
    pub plan: Vec<Oid>,
    pub proposals: Vec<Oid>,
    pub decisions: Vec<String>,
    pub tasks: Vec<TaskState>,
    pub closed_at: Option<Timestamp>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskState {
    pub oid: Oid,
    pub stalls: u32,
    pub carried_forward: bool,
}

impl Day {
    pub fn open(day: Date) -> Day {
        Day {
            day,
            plan: Vec::new(),
            proposals: Vec::new(),
            decisions: Vec::new(),
            tasks: Vec::new(),
            closed_at: None,
        }
    }
}
