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

/// Untagged on the wire, but dispatched by hand on read: `PullResponse` and
/// `PushResponse` default every field, so the derived untagged deserializer
/// would accept any object as a `Pull` before ever trying the later variants.
/// Deciding by which key is present, instead of by which variant happens to
/// parse first, also lets a helper add a field to its response without
/// breaking `dam`. A response matching no variant's keys is an error rather
/// than the last variant tried, and `error` is looked for first, so a
/// response carrying both `error` and a shape's own keys reads as a failure.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum Response {
    Capabilities(Capabilities),
    Pull(PullResponse),
    Push(PushResponse),
    Error { error: String },
}

impl Response {
    /// The shape's own name, for a message about a response of the wrong
    /// shape. The values a response carries are the operator's private task
    /// text and never belong in an error.
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
        if PULL_KEYS.iter().any(|key| object.contains_key(*key)) {
            return serde_json::from_value(value)
                .map(Response::Pull)
                .map_err(D::Error::custom);
        }
        // The count rather than the keys: what a helper sent is its own text,
        // and an error carrying it reaches the operator's terminal.
        Err(D::Error::custom(format!(
            "a response naming none of objects, removed, cancelled or sync, over {} keys",
            object.len()
        )))
    }
}

/// A pull response is recognized by carrying at least one of these. Every
/// field defaults, so without this any JSON object would read as an empty
/// pull: the remote is empty and removed nothing.
const PULL_KEYS: [&str; 4] = ["objects", "removed", "cancelled", "sync"];

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
    /// Stable across every resend of this same mutation, so a remote that
    /// deduplicates by key does the work once however often an interrupted
    /// push repeats it. A helper that turns one mutation into several remote
    /// commands numbers them in the key's last character, which dam leaves
    /// free.
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
