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

#[test]
fn a_call_that_never_reached_the_service_says_it_is_unreachable() {
    let never_reached = [
        ureq::Error::HostNotFound,
        ureq::Error::ConnectionFailed,
        ureq::Error::Io(std::io::ErrorKind::ConnectionRefused.into()),
        ureq::Error::Io(std::io::ErrorKind::NetworkUnreachable.into()),
        ureq::Error::Io(std::io::ErrorKind::HostUnreachable.into()),
        ureq::Error::Timeout(ureq::Timeout::Resolve),
        ureq::Error::Timeout(ureq::Timeout::Connect),
    ];
    for e in never_reached {
        let shown = e.to_string();
        let err = call_failed("Todoist", e);
        assert!(
            matches!(err, ServiceError::Unreachable { .. }),
            "{shown}: {err:?}"
        );
        assert!(err.to_string().starts_with("Todoist could not be reached"));
    }
}

#[test]
fn a_call_that_ran_out_of_time_says_so() {
    let err = call_failed("Todoist", ureq::Error::Timeout(ureq::Timeout::Global));
    assert!(matches!(err, ServiceError::TimedOut { .. }), "{err:?}");
    assert_eq!(err.to_string(), "Todoist did not answer within 30s");
}

#[test]
fn a_connection_lost_after_it_was_made_is_a_transport_failure() {
    for kind in [
        std::io::ErrorKind::ConnectionReset,
        std::io::ErrorKind::UnexpectedEof,
        std::io::ErrorKind::BrokenPipe,
    ] {
        let err = call_failed("Todoist", ureq::Error::Io(kind.into()));
        assert!(matches!(err, ServiceError::Transport { .. }), "{err:?}");
    }
}
