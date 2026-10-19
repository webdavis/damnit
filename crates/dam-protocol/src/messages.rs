use serde::{Deserialize, Deserializer, Serialize};

use crate::wire::WireObject;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "lowercase")]
pub enum Request {
    Capabilities,
    Pull { since: Option<String> },
    Push { mutations: Vec<Mutation> },
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "PullResponseDoc")]
pub struct PullResponse {
    pub objects: Vec<WireObject>,
    #[serde(skip)]
    pub unreadable: Vec<UnreadableObject>,
    pub removed: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cancelled: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnreadableObject {
    pub remote_id: String,
    pub why: String,
}

#[derive(Deserialize)]
struct PullResponseDoc {
    objects: Option<Vec<serde_json::Value>>,
    removed: Option<Vec<String>>,
    cancelled: Option<Vec<String>>,
    #[serde(default, deserialize_with = "present")]
    sync: Option<Option<String>>,
    #[serde(flatten)]
    other: serde_json::Map<String, serde_json::Value>,
}

fn present<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Option<String>>, D::Error> {
    Option::deserialize(d).map(Some)
}

impl TryFrom<PullResponseDoc> for PullResponse {
    type Error = String;

    fn try_from(doc: PullResponseDoc) -> Result<PullResponse, String> {
        if doc.objects.is_none()
            && doc.removed.is_none()
            && doc.cancelled.is_none()
            && doc.sync.is_none()
        {
            return Err(format!(
                "a pull answer naming none of objects, removed, cancelled or sync, over {} keys",
                doc.other.len()
            ));
        }
        let mut response = PullResponse {
            removed: doc.removed.unwrap_or_default(),
            cancelled: doc.cancelled.unwrap_or_default(),
            sync: doc.sync.flatten(),
            ..PullResponse::default()
        };
        for value in doc.objects.unwrap_or_default() {
            let remote_id = value["remote_id"].as_str().unwrap_or_default().to_string();
            match unreadable::read_object(&value) {
                Ok(object) => response.objects.push(object),
                Err(why) => response
                    .unreadable
                    .push(UnreadableObject { remote_id, why }),
            }
        }
        Ok(response)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PushResponse {
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

mod unreadable;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod pull_tests;
