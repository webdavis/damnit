use super::*;
use crate::Reply;

#[test]
fn a_response_naming_no_known_shape_is_refused_rather_than_read_as_an_empty_pull() {
    for text in ["{}", r#"{"foo":1}"#, r#"{"objectss":[]}"#] {
        let err = serde_json::from_str::<Reply<PullResponse>>(text).unwrap_err();
        assert!(
            err.to_string()
                .contains("objects, removed, cancelled or sync"),
            "{text}: {err}"
        );
    }
}

#[test]
fn a_refusal_of_an_unknown_shape_counts_its_keys_without_quoting_them() {
    let err = serde_json::from_str::<Reply<PullResponse>>(r#"{"body":"buy oat milk","x":1}"#)
        .unwrap_err();
    let text = err.to_string();
    assert!(text.contains('2'), "{text}");
    assert!(!text.contains("oat milk"), "{text}");
    assert!(!text.contains("body"), "{text}");
}

#[test]
fn any_one_pull_key_on_its_own_is_enough_to_read_a_pull_response() {
    for text in [
        r#"{"objects":[]}"#,
        r#"{"removed":[]}"#,
        r#"{"sync":"s1"}"#,
        r#"{"sync":null}"#,
    ] {
        assert!(
            matches!(
                serde_json::from_str::<Reply<PullResponse>>(text).unwrap(),
                Reply::Answered(_)
            ),
            "{text}"
        );
    }
}

#[test]
fn a_pull_response_with_an_unknown_field_still_deserializes_as_pull() {
    let r: Reply<PullResponse> =
        serde_json::from_str(r#"{"objects":[],"removed":[],"cursor":"abc"}"#).unwrap();
    assert_eq!(r, Reply::Answered(PullResponse::default()));
}

#[test]
fn a_pull_carrying_only_cancellations_is_a_pull() {
    let read = serde_json::from_str::<PullResponse>(r#"{"cancelled":["primary/e1"]}"#).unwrap();
    assert_eq!(read.cancelled, vec!["primary/e1".to_string()]);
}

#[test]
fn pull_with_cancellations_round_trips_the_fixture() {
    let text = include_str!("../../fixtures/pull-cancelled.response.json").trim_end();
    let read: Reply<PullResponse> = serde_json::from_str(text).unwrap();
    assert_eq!(serde_json::to_string(&read).unwrap(), text);
}
