use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

use crate::Capabilities;
use crate::wire::WireObject;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "lowercase")]
pub enum Request {
    Capabilities,
    Pull { since: Option<String> },
    Push { mutations: Vec<Mutation> },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum Response {
    Capabilities(Capabilities),
    Pull(PullResponse),
    Push(PushResponse),
    Error { error: String },
}

impl Response {
    pub fn shape(&self) -> &'static str {
        match self {
            Response::Capabilities(_) => "capabilities",
            Response::Pull(_) => "pull",
            Response::Push(_) => "push",
            Response::Error { .. } => "error",
        }
    }
}

#[derive(Deserialize)]
struct ErrorPayload {
    error: String,
}

impl<'de> Deserialize<'de> for Response {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let object = value
            .as_object()
            .ok_or_else(|| D::Error::custom("expected a JSON object"))?;
        if object.contains_key("error") {
            let payload: ErrorPayload = serde_json::from_value(value).map_err(D::Error::custom)?;
            return Ok(Response::Error {
                error: payload.error,
            });
        }
        if object.contains_key("results") {
            return serde_json::from_value(value)
                .map(Response::Push)
                .map_err(D::Error::custom);
        }
        if object.contains_key("protocol") {
            return serde_json::from_value(value)
                .map(Response::Capabilities)
                .map_err(D::Error::custom);
        }
        if KEYS_THAT_MARK_A_PULL_RESPONSE
            .iter()
            .any(|key| object.contains_key(*key))
        {
            return serde_json::from_value(value)
                .map(Response::Pull)
                .map_err(D::Error::custom);
        }
        Err(D::Error::custom(format!(
            "a response naming none of objects, removed, cancelled or sync, over {} keys",
            object.len()
        )))
    }
}

const KEYS_THAT_MARK_A_PULL_RESPONSE: [&str; 4] = ["objects", "removed", "cancelled", "sync"];

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PullResponse {
    #[serde(default)]
    pub objects: Vec<WireObject>,
    #[serde(default)]
    pub removed: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cancelled: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sync: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PushResponse {
    #[serde(default)]
    pub results: Vec<MutationResult>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mutation {
    pub op: String,
    pub oid: String,
    pub idempotency_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: Option<WireObject>,
    #[serde(default)]
    pub fields: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutationResult {
    pub oid: String,
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub why: Option<String>,
}

#[cfg(test)]
mod tests;
