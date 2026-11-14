use super::*;
use crate::wire::{WireObject, WireTask};
use crate::{Capabilities, Reply};

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
            estimate: None,
            focus: None,
        }),
        event: None,
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
        removes_only_empty_containers: true,
    };
    let answer = Reply::Answered(caps);
    assert_eq!(
        serde_json::to_string(&answer).unwrap(),
        golden("capabilities.response")
    );
    assert_eq!(
        serde_json::from_str::<Reply<Capabilities>>(&golden("capabilities.response")).unwrap(),
        answer
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
        ..PullResponse::default()
    };
    let answer = Reply::Answered(res);
    assert_eq!(
        serde_json::to_string(&answer).unwrap(),
        golden("pull.response")
    );
    assert_eq!(
        serde_json::from_str::<Reply<PullResponse>>(&golden("pull.response")).unwrap(),
        answer
    );
}

#[test]
fn push_round_trips_the_fixtures() {
    let m = Mutation {
        op: "update".into(),
        oid: oid(),
        idempotency_key: "0102030a-0b0c-4d0e-8f10-111213141500".into(),
        remote_id: Some("r1".into()),
        container_remote_id: None,
        object: Some(milk("buy oat milk", true)),
        fields: vec!["subject".into(), "done".into()],
    };
    let req = serde_json::to_string(&Request::Push {
        mutations: vec![m],
        early_answers: false,
    });
    assert_eq!(req.unwrap(), golden("push.request"));
    let res = PushResponse {
        results: vec![MutationResult {
            oid: oid(),
            verdict: Verdict::Accepted {
                remote_id: Some("r1".into()),
            },
        }],
        stopped: None,
    };
    let answer = Reply::Answered(res);
    assert_eq!(
        serde_json::to_string(&answer).unwrap(),
        golden("push.response")
    );
    assert_eq!(
        serde_json::from_str::<Reply<PushResponse>>(&golden("push.response")).unwrap(),
        answer
    );
}

#[test]
fn a_push_answer_without_results_is_refused_rather_than_read_as_no_answers() {
    let err = serde_json::from_str::<Reply<PushResponse>>(r#"{"objects":[]}"#).unwrap_err();
    assert!(err.to_string().contains("results"), "{err}");
}

#[test]
fn an_attendee_writes_self_only_when_true_and_reads_its_absence_as_false() {
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

#[test]
fn a_mutation_names_the_container_it_files_into_only_when_given_one() {
    let without: Request = serde_json::from_str(&golden("push.request")).unwrap();
    let Request::Push { mutations, .. } = without else {
        panic!("not a push");
    };
    assert_eq!(mutations[0].container_remote_id, None);
    let mut named = mutations[0].clone();
    named.container_remote_id = Some("i:r2".into());
    let wire = serde_json::to_value(&named).unwrap();
    assert_eq!(wire["container_remote_id"], "i:r2");
    assert_eq!(serde_json::from_value::<Mutation>(wire).unwrap(), named);
}
