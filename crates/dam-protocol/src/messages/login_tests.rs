use super::*;
use crate::Reply;

fn golden(text: &str) -> String {
    text.trim_end().to_string()
}

#[test]
fn a_login_request_matches_the_fixture() {
    assert_eq!(
        serde_json::to_string(&Request::Login).unwrap(),
        golden(include_str!("../../fixtures/login.request.json"))
    );
}

#[test]
fn a_login_answer_round_trips_the_fixture() {
    let fixture = golden(include_str!("../../fixtures/login.response.json"));
    let answer = Reply::Answered(LoginResponse { signed_in: true });
    assert_eq!(serde_json::to_string(&answer).unwrap(), fixture);
    assert_eq!(
        serde_json::from_str::<Reply<LoginResponse>>(&fixture).unwrap(),
        answer
    );
}

#[test]
fn a_login_answer_without_signed_in_is_not_an_answer() {
    assert!(serde_json::from_str::<Reply<LoginResponse>>("{}").is_err());
}

#[test]
fn a_refused_login_is_an_error_object() {
    let refused: Reply<LoginResponse> =
        serde_json::from_str(r#"{"error":"todoist answered 401: Unauthorized"}"#).unwrap();
    assert!(matches!(refused, Reply::Failed { error } if error.contains("401")));
}
