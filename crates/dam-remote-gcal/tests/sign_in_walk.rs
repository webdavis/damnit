mod loopback;
mod support;

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;

use dam_remote_gcal::{Client, Endpoints, SignIn, SignInError};
use loopback::Reply;

const CLIENT_SECRET: &str = "GOCSPX-SUPERSECRETCLIENT";
const REFRESH: &str = "1//0gSUPERSECRETREFRESH";

fn client() -> Client {
    Client {
        id: "123.apps.googleusercontent.com".into(),
        secret: CLIENT_SECRET.into(),
    }
}

/// The value of one query parameter in the announced URL.
fn param(url: &str, name: &str) -> String {
    let query = url.split_once('?').map(|(_, q)| q).unwrap_or_default();
    let raw = query
        .split('&')
        .find_map(|p| p.strip_prefix(&format!("{name}=")))
        .unwrap_or_default();
    raw.replace("%3A", ":").replace("%2F", "/")
}

/// Plays the browser: opens the redirect with the given query, returns the page.
fn browse(url: &str, query: impl Fn(&str) -> String) -> std::thread::JoinHandle<String> {
    let redirect = param(url, "redirect_uri");
    let state = param(url, "state");
    let address = redirect.trim_start_matches("http://").to_string();
    let line = format!(
        "GET /?{} HTTP/1.1\r\nHost: {address}\r\n\r\n",
        query(&state)
    );
    std::thread::spawn(move || {
        let mut stream = TcpStream::connect(address).unwrap();
        stream.write_all(line.as_bytes()).unwrap();
        let mut page = String::new();
        let _ = stream.read_to_string(&mut page);
        page
    })
}

#[test]
fn a_granted_consent_is_exchanged_for_the_refresh_token() {
    let _guard = support::guard("a_granted_consent_is_exchanged_for_the_refresh_token");
    let mut routes = HashMap::new();
    routes.insert("POST /token", vec![Reply::json(200, serde_json::json!({
        "access_token": "ya29.ACCESS", "expires_in": 3599, "refresh_token": REFRESH,
        "scope": "https://www.googleapis.com/auth/calendar.events.readonly", "token_type": "Bearer"
    }))]);
    let google = loopback::serve(routes);
    let sign_in = SignIn::new(Endpoints::loopback(&google.base).unwrap());
    let mut browser = None;
    let token = sign_in
        .mint(&client(), &mut |url| {
            browser = Some(browse(url, |state| {
                format!("state={state}&code=4%2F0AbCODE&scope=x")
            }));
        })
        .unwrap();
    assert_eq!(token.expose(), REFRESH);
    let page = browser.unwrap().join().unwrap();
    assert!(page.contains("200"), "{page}");
    let seen = google.seen();
    assert_eq!(seen.len(), 1);
    let body = &seen[0].body;
    for field in [
        "grant_type=authorization_code",
        "code=4%2F0AbCODE",
        "client_id=123.apps.googleusercontent.com",
        "client_secret=GOCSPX-SUPERSECRETCLIENT",
        "code_verifier=",
    ] {
        assert!(body.contains(field), "{field} missing from the exchange");
    }
    assert!(
        body.contains("redirect_uri=http%3A%2F%2F127.0.0.1%3A"),
        "{body}"
    );
}

#[test]
fn a_refused_exchange_names_the_status_and_googles_word_and_quotes_nothing() {
    let _guard =
        support::guard("a_refused_exchange_names_the_status_and_googles_word_and_quotes_nothing");
    let mut routes = HashMap::new();
    routes.insert(
        "POST /token",
        vec![Reply::json(
            401,
            serde_json::json!({
                "error": "invalid_client", "error_description": format!("echo {CLIENT_SECRET}")
            }),
        )],
    );
    let google = loopback::serve(routes);
    let sign_in = SignIn::new(Endpoints::loopback(&google.base).unwrap());
    let mut browser = None;
    let err = sign_in
        .mint(&client(), &mut |url| {
            browser = Some(browse(url, |state| format!("state={state}&code=c")));
        })
        .unwrap_err();
    let _ = browser.unwrap().join();
    assert_eq!(
        err,
        SignInError::Exchange {
            status: Some(401),
            code: Some("invalid_client")
        }
    );
    let said = err.to_string();
    assert!(
        said.contains("401") && said.contains("invalid_client"),
        "{said}"
    );
    assert!(!said.contains(CLIENT_SECRET), "{said}");
}

#[test]
fn an_exchange_answering_no_refresh_token_is_refused_by_name() {
    let _guard = support::guard("an_exchange_answering_no_refresh_token_is_refused_by_name");
    for answered in [
        serde_json::json!({"access_token": "ya29.A", "expires_in": 3599}),
        serde_json::json!({"access_token": "ya29.A", "refresh_token": ""}),
    ] {
        let mut routes = HashMap::new();
        routes.insert("POST /token", vec![Reply::json(200, answered.clone())]);
        let google = loopback::serve(routes);
        let sign_in = SignIn::new(Endpoints::loopback(&google.base).unwrap());
        let mut browser = None;
        let err = sign_in
            .mint(&client(), &mut |url| {
                browser = Some(browse(url, |state| format!("state={state}&code=c")));
            })
            .unwrap_err();
        let _ = browser.unwrap().join();
        assert_eq!(err, SignInError::NoRefreshToken, "{answered}");
    }
}

#[test]
fn a_consent_refused_in_the_browser_sends_no_exchange() {
    let _guard = support::guard("a_consent_refused_in_the_browser_sends_no_exchange");
    let google = loopback::serve(HashMap::new());
    let sign_in = SignIn::new(Endpoints::loopback(&google.base).unwrap());
    let mut browser = None;
    let err = sign_in
        .mint(&client(), &mut |url| {
            browser = Some(browse(url, |state| {
                format!("error=access_denied&state={state}")
            }));
        })
        .unwrap_err();
    let page = browser.unwrap().join().unwrap();
    assert_eq!(err, SignInError::Denied("access_denied"));
    assert!(page.contains("refused"), "{page}");
    assert!(google.seen().is_empty());
}
