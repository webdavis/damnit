use dam_domain::{Day, Oid, TaskState};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DayDoc {
    pub day: String,
    #[serde(default)]
    pub plan: Vec<String>,
    #[serde(default)]
    pub proposals: Vec<String>,
    #[serde(default)]
    pub decisions: Vec<String>,
    #[serde(default)]
    pub tasks: Vec<TaskStateDoc>,
    #[serde(default)]
    pub closed_at: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskStateDoc {
    pub oid: String,
    #[serde(default)]
    pub stalls: u32,
    #[serde(default)]
    pub carried_forward: bool,
}

pub fn day_to_doc(day: &Day) -> DayDoc {
    let oids = |list: &[Oid]| list.iter().map(ToString::to_string).collect();
    DayDoc {
        day: day.day.to_string(),
        plan: oids(&day.plan),
        proposals: oids(&day.proposals),
        decisions: day.decisions.clone(),
        tasks: day
            .tasks
            .iter()
            .map(|t| TaskStateDoc {
                oid: t.oid.to_string(),
                stalls: t.stalls,
                carried_forward: t.carried_forward,
            })
            .collect(),
        closed_at: day.closed_at.map(|at| at.to_string()),
    }
}

pub fn day_from_doc(doc: DayDoc) -> Result<Day, String> {
    let oid = |text: &str| Oid::parse(text).map_err(|e| format!("{text:?}: {e}"));
    let oids = |list: &[String]| list.iter().map(|o| oid(o)).collect::<Result<Vec<_>, _>>();
    Ok(Day {
        day: doc
            .day
            .parse()
            .map_err(|_| format!("day {:?} is not YYYY-MM-DD", doc.day))?,
        plan: oids(&doc.plan)?,
        proposals: oids(&doc.proposals)?,
        decisions: doc.decisions,
        tasks: doc
            .tasks
            .iter()
            .map(|t| {
                Ok(TaskState {
                    oid: oid(&t.oid)?,
                    stalls: t.stalls,
                    carried_forward: t.carried_forward,
                })
            })
            .collect::<Result<_, String>>()?,
        closed_at: doc
            .closed_at
            .map(|at| {
                at.parse()
                    .map_err(|_| format!("closed_at {at:?} is not an RFC 3339 time"))
            })
            .transpose()?,
    })
}
