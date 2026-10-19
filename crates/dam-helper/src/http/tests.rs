use super::*;

fn answer(status: u16, body: &str) -> ureq::http::Response<ureq::Body> {
    ureq::http::Response::builder()
        .status(status)
        .header("Retry-After", "42")
        .body(ureq::Body::builder().data(body))
        .unwrap()
}

#[test]
fn the_body_ceiling_is_the_protocol_line_not_ureqs_narrower_default() {
    assert_eq!(
        LONGEST_ANSWER_ONE_PROTOCOL_LINE_CAN_DELIVER,
        dam_protocol::MAX_LINE
    );
}

#[test]
fn a_body_past_the_ceiling_is_its_own_refusal_naming_the_ceiling_not_a_transport_failure() {
    let err = read_within("Todoist", answer(200, r#"{"items":[1,2,3]}"#), 8).unwrap_err();
    assert!(
        matches!(
            err,
            ServiceError::TooLarge {
                service: "Todoist",
                limit: 8
            }
        ),
        "{err:?}"
    );
    assert!(err.to_string().contains('8'), "{err}");
}

#[test]
fn a_body_inside_the_ceiling_is_handed_back() {
    assert_eq!(
        read_answer("Todoist", answer(200, r#"{"ok":true}"#)).unwrap(),
        r#"{"ok":true}"#
    );
}
