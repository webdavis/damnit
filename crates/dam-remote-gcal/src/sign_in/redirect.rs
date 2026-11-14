use std::io::{BufRead, Read, Write};
use std::net::TcpListener;
use std::time::Duration;

use super::SignInError;
use crate::Endpoints;
use crate::encoding::{form, percent_decoded};

const READ_ONLY_SCOPE_SO_GOOGLE_REFUSES_ANY_WRITE: &str =
    "https://www.googleapis.com/auth/calendar.events.readonly";
const OFFLINE_CONSENT_THAT_YIELDS_A_REFRESH_TOKEN: [(&str, &str); 2] =
    [("access_type", "offline"), ("prompt", "consent")];
const REDIRECT_READ_DEADLINE_ONCE_THE_BROWSER_CONNECTS: Duration = Duration::from_secs(30);
const REDIRECT_REQUEST_READ_MAX_BYTES: u64 = 8 * 1024;
const GOOGLE_WORDS_FOR_A_REFUSED_CONSENT: [&str; 5] = [
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
    let [access_type, prompt] = OFFLINE_CONSENT_THAT_YIELDS_A_REFRESH_TOKEN;
    let query = form(&[
        ("client_id", client_id),
        ("redirect_uri", redirect_uri),
        ("response_type", "code"),
        ("scope", READ_ONLY_SCOPE_SO_GOOGLE_REFUSES_ANY_WRITE),
        ("code_challenge", challenge),
        ("code_challenge_method", "S256"),
        ("state", state),
        access_type,
        prompt,
    ]);
    format!("{}?{query}", endpoints.authorization)
}

pub(super) fn await_the_one_browser_redirect(
    listener: &TcpListener,
    state: &str,
) -> Result<String, SignInError> {
    let (mut stream, _) = listener.accept().map_err(|_| SignInError::Redirect)?;
    stream
        .set_read_timeout(Some(REDIRECT_READ_DEADLINE_ONCE_THE_BROWSER_CONNECTS))
        .map_err(|_| SignInError::Redirect)?;
    let mut line = String::new();
    std::io::BufReader::new((&stream).take(REDIRECT_REQUEST_READ_MAX_BYTES))
        .read_line(&mut line)
        .map_err(|_| SignInError::Redirect)?;
    let answered = code_of(&line, state);
    let _ = stream.write_all(page_telling_the_browser_the_outcome(answered.is_ok()).as_bytes());
    answered
}

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
        let word = GOOGLE_WORDS_FOR_A_REFUSED_CONSENT
            .into_iter()
            .find(|d| *d == error)
            .unwrap_or("an error Google did not name");
        return Err(SignInError::Denied(word));
    }
    stated("code")
        .filter(|code| !code.is_empty())
        .ok_or(SignInError::NoCode)
}

fn page_telling_the_browser_the_outcome(granted: bool) -> String {
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
mod tests;
