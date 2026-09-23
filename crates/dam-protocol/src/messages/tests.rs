use super::*;
use crate::Capabilities;
use crate::wire::{WireObject, WireTask};

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
            completed_at: None,
            priority: 4,
            due: Some("2026-09-25".into()),
            deadline: None,
            event: None,
        }),
        event: None,
    }
}

#[test]
fn a_response_naming_no_known_shape_is_refused_rather_than_read_as_an_empty_pull() {
    for text in ["{}", r#"{"foo":1}"#, r#"{"objectss":[]}"#] {
        let err = serde_json::from_str::<Response>(text).unwrap_err();
        assert!(
            err.to_string()
                .contains("objects, removed, cancelled or sync"),
            "{text}: {err}"
        );
    }
}

#[test]
fn a_refusal_of_an_unknown_shape_counts_its_keys_without_quoting_them() {
    let err = serde_json::from_str::<Response>(r#"{"body":"buy oat milk","x":1}"#).unwrap_err();
    let text = err.to_string();
    assert!(text.contains('2'), "{text}");
    assert!(!text.contains("oat milk"), "{text}");
    assert!(!text.contains("body"), "{text}");
}

#[test]
fn any_one_pull_key_on_its_own_is_enough_to_read_a_pull_response() {
    for text in [r#"{"objects":[]}"#, r#"{"removed":[]}"#, r#"{"sync":"s1"}"#] {
        assert!(
            matches!(
                serde_json::from_str::<Response>(text).unwrap(),
                Response::Pull(_)
            ),
            "{text}"
        );
    }
}

fn golden(name: &str) -> String {
    let text = match name {
        "capabilities.request" => include_str!("../../fixtures/capabilities.request.json"),
        "capabilities.response" => include_str!("../../fixtures/capabilities.response.json"),
        "pull.request" => include_str!("../../fixtures/pull.request.json"),
        "pull.response" => include_str!("../../fixtures/pull.response.json"),
        "push.request" => include_str!("../../fixtures/push.request.json"),
        "push.response" => include_str!("../../fixtures/push.response.json"),
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
        cancelled: vec![],
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
        idempotency_key: "0102030a-0b0c-4d0e-8f10-111213141500".into(),
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
            cancelled: vec![],
            sync: None,
        })
    );
}

#[test]
fn a_push_response_with_an_unknown_field_still_deserializes_as_push() {
    let r: Response = serde_json::from_str(r#"{"results":[],"cursor":"abc"}"#).unwrap();
    assert_eq!(r, Response::Push(PushResponse { results: vec![] }));
}

#[test]
fn a_pull_carrying_only_cancellations_is_a_pull() {
    let read = serde_json::from_str::<Response>(r#"{"cancelled":["primary/e1"]}"#).unwrap();
    assert!(matches!(read, Response::Pull(p) if p.cancelled == vec!["primary/e1".to_string()]));
}

#[test]
fn pull_with_cancellations_round_trips_the_fixture() {
    let text = include_str!("../../fixtures/pull-cancelled.response.json").trim_end();
    let read: Response = serde_json::from_str(text).unwrap();
    assert_eq!(serde_json::to_string(&read).unwrap(), text);
}

/// Written only when true, so every document without it reads exactly as
/// before, and a stored object from before the field reads as false.
#[test]
fn an_attendee_writes_self_only_when_it_is_the_calendars_own() {
    let own = crate::WireAttendee {
        email: "me@x".into(),
        response: "declined".into(),
        is_self: true,
    };
    assert_eq!(
        serde_json::to_string(&own).unwrap(),
        r#"{"email":"me@x","response":"declined","self":true}"#
    );
    let other = crate::WireAttendee {
        is_self: false,
        ..own.clone()
    };
    assert_eq!(
        serde_json::to_string(&other).unwrap(),
        r#"{"email":"me@x","response":"declined"}"#
    );
    let read: crate::WireAttendee =
        serde_json::from_str(r#"{"email":"me@x","response":"accepted"}"#).unwrap();
    assert!(!read.is_self);
}
