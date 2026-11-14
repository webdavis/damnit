use super::*;
use crate::Endpoints;

#[test]
fn the_authorization_url_asks_for_read_only_events_offline_with_pkce_byte_for_byte() {
    let url = authorization_url(
        &Endpoints::production(),
        "123.apps.googleusercontent.com",
        "http://127.0.0.1:5555",
        "CHALLENGE",
        "STATE",
    );
    assert_eq!(
        url,
        "https://accounts.google.com/o/oauth2/v2/auth?client_id=123.apps.googleusercontent.com\
         &redirect_uri=http%3A%2F%2F127.0.0.1%3A5555&response_type=code\
         &scope=https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fcalendar.events.readonly\
         &code_challenge=CHALLENGE&code_challenge_method=S256&state=STATE\
         &access_type=offline&prompt=consent"
    );
}

#[test]
fn the_code_is_read_once_the_state_agrees() {
    assert_eq!(
        code_of("GET /?state=S1&code=4%2F0Ab HTTP/1.1", "S1"),
        Ok("4/0Ab".to_string())
    );
}

#[test]
fn a_redirect_for_another_run_is_refused_whatever_else_it_says() {
    assert_eq!(
        code_of("GET /?state=OTHER&code=c HTTP/1.1", "S1"),
        Err(SignInError::State)
    );
    assert_eq!(
        code_of("GET /?code=c HTTP/1.1", "S1"),
        Err(SignInError::State)
    );
    assert_eq!(
        code_of("GET /?error=access_denied&state=OTHER HTTP/1.1", "S1"),
        Err(SignInError::State)
    );
}

#[test]
fn a_refusal_in_the_browser_names_googles_word_and_nothing_else() {
    assert_eq!(
        code_of("GET /?error=access_denied&state=S1 HTTP/1.1", "S1"),
        Err(SignInError::Denied("access_denied"))
    );
    assert_eq!(
        code_of("GET /?error=%3Cscript%3E&state=S1 HTTP/1.1", "S1"),
        Err(SignInError::Denied("an error Google did not name"))
    );
}

#[test]
fn a_redirect_with_no_code_is_refused() {
    assert_eq!(
        code_of("GET /?state=S1 HTTP/1.1", "S1"),
        Err(SignInError::NoCode)
    );
    assert_eq!(
        code_of("GET /?state=S1&code= HTTP/1.1", "S1"),
        Err(SignInError::NoCode)
    );
}
