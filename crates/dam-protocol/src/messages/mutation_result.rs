use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::object::read_object;

const NO_REASON_GIVEN: &str = "the remote gave no reason";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Value", into = "MutationResultDoc")]
pub struct MutationResult {
    pub oid: String,
    pub verdict: Verdict,
}

impl MutationResult {
    pub fn ok(&self) -> bool {
        matches!(self.verdict, Verdict::Accepted { .. })
    }

    pub fn remote_id(&self) -> Option<&str> {
        match &self.verdict {
            Verdict::Accepted { remote_id } => remote_id.as_deref(),
            Verdict::Refused { .. } => None,
        }
    }

    pub fn why(&self) -> Option<&str> {
        match &self.verdict {
            Verdict::Accepted { .. } => None,
            Verdict::Refused { why } => Some(why),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    Accepted { remote_id: Option<String> },
    Refused { why: String },
}

#[derive(Serialize, Deserialize)]
struct MutationResultDoc {
    oid: String,
    ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    remote_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    why: Option<String>,
}

impl From<MutationResultDoc> for MutationResult {
    fn from(doc: MutationResultDoc) -> MutationResult {
        let verdict = match doc.ok {
            true => Verdict::Accepted {
                remote_id: doc.remote_id,
            },
            false => Verdict::Refused {
                why: doc.why.unwrap_or_else(|| NO_REASON_GIVEN.to_string()),
            },
        };
        MutationResult {
            oid: doc.oid,
            verdict,
        }
    }
}

impl TryFrom<Value> for MutationResult {
    type Error = serde_json::Error;

    fn try_from(value: Value) -> Result<MutationResult, serde_json::Error> {
        read_object::<MutationResultDoc>(value).map(MutationResult::from)
    }
}

impl From<MutationResult> for MutationResultDoc {
    fn from(result: MutationResult) -> MutationResultDoc {
        let (ok, remote_id, why) = match result.verdict {
            Verdict::Accepted { remote_id } => (true, remote_id, None),
            Verdict::Refused { why } => (false, None, Some(why)),
        };
        MutationResultDoc {
            oid: result.oid,
            ok,
            remote_id,
            why,
        }
    }
}

#[cfg(test)]
mod tests;
