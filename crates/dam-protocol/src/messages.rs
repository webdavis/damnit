use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

use crate::Capabilities;

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
/// breaking `dam`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum Response {
    Capabilities(Capabilities),
    Pull(PullResponse),
    Push(PushResponse),
    Error { error: String },
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
        serde_json::from_value(value)
            .map(Response::Pull)
            .map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PullResponse {
    #[serde(default)]
    pub objects: Vec<WireObject>,
    #[serde(default)]
    pub removed: Vec<String>,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireObject {
    pub oid: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_id: Option<String>,
    pub kind: String,
    pub subject: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub depends: Vec<String>,
    #[serde(default)]
    pub reminders: Vec<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recurrence: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task: Option<WireTask>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event: Option<WireEvent>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireTask {
    pub done: bool,
    pub priority: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireEvent {
    pub start: String,
    pub end: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(default)]
    pub attendees: Vec<WireAttendee>,
    pub status: String,
    pub transparency: String,
    pub visibility: String,
    pub event_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organizer: Option<WirePerson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conference: Option<WireConference>,
    #[serde(default)]
    pub attachments: Vec<WireAttachment>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireAttendee {
    pub email: String,
    pub response: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WirePerson {
    pub email: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireConference {
    pub provider: String,
    pub url: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireAttachment {
    pub url: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Capabilities;

    fn oid() -> String {
        "01".repeat(20)
    }

    fn milk(subject: &str, done: bool) -> WireObject {
        WireObject {
            oid: oid(),
            remote_id: Some("r1".into()),
            kind: "task".into(),
            subject: subject.into(),
            body: String::new(),
            path: "inbox/".into(),
            labels: vec!["errand".into()],
            depends: vec![],
            reminders: vec![],
            recurrence: None,
            task: Some(WireTask {
                done,
                priority: 4,
                due: Some("2026-09-25".into()),
                deadline: None,
                event: None,
            }),
            event: None,
        }
    }

    fn golden(name: &str) -> String {
        let text = match name {
            "capabilities.request" => include_str!("../fixtures/capabilities.request.json"),
            "capabilities.response" => include_str!("../fixtures/capabilities.response.json"),
            "pull.request" => include_str!("../fixtures/pull.request.json"),
            "pull.response" => include_str!("../fixtures/pull.response.json"),
            "push.request" => include_str!("../fixtures/push.request.json"),
            "push.response" => include_str!("../fixtures/push.response.json"),
            _ => unreachable!(),
        };
        text.trim_end().to_string()
    }

    #[test]
    fn capabilities_request_matches_the_fixture() {
        assert_eq!(
            serde_json::to_string(&Request::Capabilities).unwrap(),
            golden("capabilities.request")
        );
    }

    #[test]
    fn capabilities_response_matches_the_fixture() {
        let caps = Capabilities {
            protocol: 1,
            kinds: vec!["task".into()],
            fields: [
                "subject",
                "body",
                "path",
                "labels",
                "priority",
                "due",
                "deadline",
                "done",
                "recurrence",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            credentials: vec!["api_token".into()],
            incremental: true,
        };
        assert_eq!(
            serde_json::to_string(&caps).unwrap(),
            golden("capabilities.response")
        );
        assert_eq!(
            serde_json::from_str::<Response>(&golden("capabilities.response")).unwrap(),
            Response::Capabilities(caps)
        );
    }

    #[test]
    fn pull_round_trips_the_fixtures() {
        let req = Request::Pull {
            since: Some("tok-1".into()),
        };
        assert_eq!(serde_json::to_string(&req).unwrap(), golden("pull.request"));
        let res = PullResponse {
            objects: vec![milk("buy milk", false)],
            removed: vec!["r9".into()],
            sync: Some("tok-2".into()),
        };
        assert_eq!(
            serde_json::to_string(&res).unwrap(),
            golden("pull.response")
        );
        assert_eq!(
            serde_json::from_str::<Response>(&golden("pull.response")).unwrap(),
            Response::Pull(res)
        );
    }

    #[test]
    fn push_round_trips_the_fixtures() {
        let m = Mutation {
            op: "update".into(),
            oid: oid(),
            remote_id: Some("r1".into()),
            object: Some(milk("buy oat milk", true)),
            fields: vec!["subject".into(), "done".into()],
        };
        let req = Request::Push { mutations: vec![m] };
        assert_eq!(serde_json::to_string(&req).unwrap(), golden("push.request"));
        let res = PushResponse {
            results: vec![MutationResult {
                oid: oid(),
                ok: true,
                remote_id: Some("r1".into()),
                why: None,
            }],
        };
        assert_eq!(
            serde_json::to_string(&res).unwrap(),
            golden("push.response")
        );
        assert_eq!(
            serde_json::from_str::<Response>(&golden("push.response")).unwrap(),
            Response::Push(res)
        );
    }

    #[test]
    fn an_error_response_deserializes() {
        let r: Response = serde_json::from_str(r#"{"error":"no such account"}"#).unwrap();
        assert_eq!(
            r,
            Response::Error {
                error: "no such account".into()
            }
        );
    }

    #[test]
    fn a_pull_response_with_an_unknown_field_still_deserializes_as_pull() {
        let r: Response =
            serde_json::from_str(r#"{"objects":[],"removed":[],"cursor":"abc"}"#).unwrap();
        assert_eq!(
            r,
            Response::Pull(PullResponse {
                objects: vec![],
                removed: vec![],
                sync: None,
            })
        );
    }

    #[test]
    fn a_push_response_with_an_unknown_field_still_deserializes_as_push() {
        let r: Response = serde_json::from_str(r#"{"results":[],"cursor":"abc"}"#).unwrap();
        assert_eq!(r, Response::Push(PushResponse { results: vec![] }));
    }
}
