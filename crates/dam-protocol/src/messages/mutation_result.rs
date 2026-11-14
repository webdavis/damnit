use serde::de::Error as _;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::object::read_object;

const NO_REASON_GIVEN: &str = "the remote gave no reason";

const NO_VERDICT: &str = "a result says ok or unanswered";

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
            Verdict::Refused { .. } | Verdict::Unanswered { .. } => None,
        }
    }

    pub fn why(&self) -> Option<&str> {
        match &self.verdict {
            Verdict::Accepted { .. } => None,
            Verdict::Refused { why } | Verdict::Unanswered { why } => Some(why),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    Accepted { remote_id: Option<String> },
    Refused { why: String },
    Unanswered { why: String },
}

#[derive(Serialize, Deserialize)]
struct MutationResultDoc {
    oid: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ok: Option<bool>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    unanswered: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    remote_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    why: Option<String>,
}

impl TryFrom<MutationResultDoc> for MutationResult {
    type Error = serde_json::Error;

    fn try_from(doc: MutationResultDoc) -> Result<MutationResult, serde_json::Error> {
        let why = || {
            doc.why
                .clone()
                .unwrap_or_else(|| NO_REASON_GIVEN.to_string())
        };
        let verdict = match (doc.unanswered, doc.ok) {
            (true, _) => Verdict::Unanswered { why: why() },
            (false, Some(true)) => Verdict::Accepted {
                remote_id: doc.remote_id.clone(),
            },
            (false, Some(false)) => Verdict::Refused { why: why() },
            (false, None) => return Err(serde_json::Error::custom(NO_VERDICT)),
        };
        Ok(MutationResult {
            oid: doc.oid,
            verdict,
        })
    }
}

impl TryFrom<Value> for MutationResult {
    type Error = serde_json::Error;

    fn try_from(value: Value) -> Result<MutationResult, serde_json::Error> {
        read_object::<MutationResultDoc>(value).and_then(MutationResult::try_from)
    }
}

impl From<MutationResult> for MutationResultDoc {
    fn from(result: MutationResult) -> MutationResultDoc {
        let (ok, unanswered, remote_id, why) = match result.verdict {
            Verdict::Accepted { remote_id } => (Some(true), false, remote_id, None),
            Verdict::Refused { why } => (Some(false), false, None, Some(why)),
            Verdict::Unanswered { why } => (None, true, None, Some(why)),
        };
        MutationResultDoc {
            oid: result.oid,
            ok,
            unanswered,
            remote_id,
            why,
        }
    }
}

#[cfg(test)]
mod tests;
