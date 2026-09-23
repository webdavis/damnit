//! The consent URL, and the single browser redirect that answers it.

use std::io::{BufRead, Read, Write};
use std::net::TcpListener;
use std::time::Duration;

use super::SignInError;
use crate::Endpoints;
use crate::encoding::{form, percent_decoded};

/// The one scope asked for. Read-only, so Google refuses a write whatever
/// code tried one.
pub(super) const SCOPE: &str = "https://www.googleapis.com/auth/calendar.events.readonly";
/// How long the redirect may take once the browser has connected.
const REDIRECT_READ_DEADLINE: Duration = Duration::from_secs(30);
/// The most of the browser's request read; a redirect line is hundreds of bytes.
const REDIRECT_READ_MAX: u64 = 8 * 1024;
/// Google's words for a refusal on the redirect.
const DENIALS: [&str; 5] = [
    "access_denied",
    "invalid_scope",
    "invalid_request",
    "unauthorized_client",
    "server_error",
];

pub(super) fn authorization_url(
    endpoints: &Endpoints,
    client_id: &str,
    redirect_uri: &str,
    challenge: &str,
    state: &str,
) -> String {
    let query = form(&[
        ("client_id", client_id),
        ("redirect_uri", redirect_uri),
        ("response_type", "code"),
        ("scope", SCOPE),
        ("code_challenge", challenge),
        ("code_challenge_method", "S256"),
        ("state", state),
        ("access_type", "offline"),
        ("prompt", "consent"),
    ]);
    format!("{}?{query}", endpoints.authorization)
}

/// One connection and one only: the operator is sitting at this walk, so the
/// listener waits for the browser they were told to open, and the browser is
/// told the outcome either way.
pub(super) fn await_code(listener: &TcpListener, state: &str) -> Result<String, SignInError> {
    let (mut stream, _) = listener.accept().map_err(|_| SignInError::Redirect)?;
    stream
        .set_read_timeout(Some(REDIRECT_READ_DEADLINE))
        .map_err(|_| SignInError::Redirect)?;
    let mut line = String::new();
    std::io::BufReader::new((&stream).take(REDIRECT_READ_MAX))
        .read_line(&mut line)
        .map_err(|_| SignInError::Redirect)?;
    let answered = code_of(&line, state);
    let _ = stream.write_all(page(answered.is_ok()).as_bytes());
    answered
}

/// The state is checked before the error: a redirect that does not carry
/// this run's state is answered as not ours, whatever else it says.
pub(super) fn code_of(request_line: &str, state: &str) -> Result<String, SignInError> {
    let query = request_line
        .split_whitespace()
        .nth(1)
        .and_then(|target| target.split_once('?'))
        .map(|(_, q)| q)
        .unwrap_or_default();
    let stated = |name: &str| {
        query
            .split('&')
            .filter_map(|pair| pair.split_once('='))
            .find(|(key, _)| *key == name)
            .map(|(_, value)| percent_decoded(value))
    };
    if stated("state").as_deref() != Some(state) {
        return Err(SignInError::State);
    }
    if let Some(error) = stated("error") {
        let word = DENIALS
            .into_iter()
            .find(|d| *d == error)
            .unwrap_or("an error Google did not name");
        return Err(SignInError::Denied(word));
    }
    stated("code")
        .filter(|code| !code.is_empty())
        .ok_or(SignInError::NoCode)
}

fn page(granted: bool) -> String {
    let body = if granted {
        "dam has the consent. Close this window and read the terminal."
    } else {
        "dam refused this redirect. Close this window and read the terminal."
    };
    format!(
        "HTTP/1.1 200 OK\r\ncontent-type: text/plain; charset=utf-8\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
        body.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Endpoints;

    /// Pinned byte for byte: a scope, a parameter or an encoding that moves is
    /// a consent screen asking for something else.
    #[test]
    fn the_authorization_url_asks_for_read_only_events_offline_with_pkce() {
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
    fn a_redirect_for_another_run_is_refused() {
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

    /// Google's own word for a refusal is a fixed vocabulary, so it is named;
    /// anything else in `error` is not quoted.
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
}
