mod loopback;
mod support;

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;

use dam_remote_gcal::{Client, Endpoints, SignIn, SignInError};
use loopback::Reply;
use sha2::Digest;

const CLIENT_SECRET: &str = "GOCSPX-SUPERSECRETCLIENT";
const REFRESH: &str = "1//0gSUPERSECRETREFRESH";

fn client() -> Client {
    Client {
        id: "123.apps.googleusercontent.com".into(),
        secret: CLIENT_SECRET.into(),
    }
}

/// The announced URL's query.
fn query_of(url: &str) -> &str {
    url.split_once('?').map(|(_, q)| q).unwrap_or_default()
}

/// One field as it travelled, still percent-encoded.
fn raw<'a>(text: &'a str, name: &str) -> Option<&'a str> {
    text.split('&')
        .find_map(|pair| pair.strip_prefix(name)?.strip_prefix('='))
}

/// RFC 7636's S256, computed independently of the code under test.
fn s256(verifier: &str) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let digest = sha2::Sha256::digest(verifier.as_bytes());
    let mut out = String::new();
    for chunk in digest.chunks(3) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0u32, |n, (i, b)| n | u32::from(*b) << (16 - 8 * i));
        for i in 0..=chunk.len() {
            out.push(ALPHABET[(n >> (18 - 6 * i) & 63) as usize] as char);
        }
    }
    out
}

/// Plays the browser: opens the redirect with the given query, returns the page.
fn browse(url: &str, query: impl Fn(&str) -> String) -> std::thread::JoinHandle<String> {
    let asked = loopback::fields(query_of(url));
    let field = |name: &str| asked.get(name).cloned().unwrap_or_default();
    let address = field("redirect_uri")
        .trim_start_matches("http://")
        .to_string();
    let line = format!(
        "GET /?{} HTTP/1.1\r\nHost: {address}\r\n\r\n",
        query(&field("state"))
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
    let mut announced = String::new();
    let token = sign_in
        .mint(&client(), &mut |url| {
            announced = url.to_string();
            browser = Some(browse(url, |state| {
                format!("state={state}&code=4%2F0AbCODE&scope=x")
            }));
        })
        .unwrap();
    assert_eq!(token.expose(), REFRESH);
    let page = browser.unwrap().join().unwrap();
    assert!(page.contains("has the consent"), "{page}");
    let seen = google.seen();
    assert_eq!(seen.len(), 1);
    assert_eq!(
        (seen[0].method.as_str(), seen[0].path.as_str()),
        ("POST", "/token")
    );
    let asked = loopback::fields(query_of(&announced));
    let sent = |name: &str| seen[0].form.get(name).cloned();
    assert_eq!(sent("grant_type").as_deref(), Some("authorization_code"));
    assert_eq!(sent("code").as_deref(), Some("4/0AbCODE"));
    assert_eq!(
        sent("client_id").as_deref(),
        Some("123.apps.googleusercontent.com")
    );
    assert_eq!(sent("client_secret").as_deref(), Some(CLIENT_SECRET));
    assert_eq!(sent("redirect_uri"), asked.get("redirect_uri").cloned());
    assert_eq!(
        raw(&seen[0].body, "redirect_uri"),
        raw(query_of(&announced), "redirect_uri"),
        "the exchange's redirect_uri must be the announced one, byte for byte"
    );
    let verifier = sent("code_verifier").unwrap_or_default();
    assert_eq!(
        Some(s256(&verifier)),
        asked.get("code_challenge").cloned(),
        "the verifier sent does not answer the challenge announced"
    );
    assert_eq!(seen[0].form.len(), 6, "{:?}", seen[0].form.keys());
}

/// The helper above is itself pinned to RFC 7636 appendix B.
#[test]
fn the_tests_own_s256_is_the_rfc_example() {
    assert_eq!(
        s256("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
        "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
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
fn an_exchange_answer_that_is_not_json_is_reported_unreadable() {
    let _guard = support::guard("an_exchange_answer_that_is_not_json_is_reported_unreadable");
    let mut routes = HashMap::new();
    let page = Reply {
        status: 200,
        body: "<html>maintenance</html>".into(),
        headers: vec![],
    };
    routes.insert("POST /token", vec![page]);
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
            status: None,
            code: None
        }
    );
    assert!(err.to_string().contains("could not be read"), "{err}");
}

/// A redirect is how a credential reaches a host nobody meant, so the token
/// endpoint's redirect is its answer and is never followed.
#[test]
fn a_redirect_from_the_token_endpoint_is_not_followed() {
    let _guard = support::guard("a_redirect_from_the_token_endpoint_is_not_followed");
    let mut elsewhere_routes = HashMap::new();
    elsewhere_routes.insert(
        "POST /token",
        vec![Reply::json(
            200,
            serde_json::json!({"refresh_token": REFRESH}),
        )],
    );
    let elsewhere = loopback::serve(elsewhere_routes);
    let mut routes = HashMap::new();
    routes.insert(
        "POST /token",
        vec![
            Reply::json(307, serde_json::json!({}))
                .with_header("Location", &format!("{}/token", elsewhere.base)),
        ],
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
            status: Some(307),
            code: None
        }
    );
    assert!(
        elsewhere.seen().is_empty(),
        "the exchange followed the redirect"
    );
}

/// The state is what ties a redirect to this walk, and the verifier is what
/// ties the exchange to it, so each walk draws both afresh.
#[test]
fn each_walk_announces_its_own_state_and_challenge() {
    let _guard = support::guard("each_walk_announces_its_own_state_and_challenge");
    let google = loopback::serve(HashMap::new());
    let sign_in = SignIn::new(Endpoints::loopback(&google.base).unwrap());
    let mut announced = Vec::new();
    for _ in 0..2 {
        let mut browser = None;
        let _ = sign_in.mint(&client(), &mut |url| {
            announced.push(loopback::fields(query_of(url)));
            browser = Some(browse(url, |state| {
                format!("error=access_denied&state={state}")
            }));
        });
        let _ = browser.map(|b| b.join());
    }
    for name in ["state", "code_challenge"] {
        let [first, second] = [0, 1].map(|n| announced[n].get(name).cloned().unwrap_or_default());
        for value in [&first, &second] {
            assert_eq!(value.len(), 43, "{name}: {value}");
            assert!(
                value
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_'),
                "{name}: {value}"
            );
        }
        assert_ne!(first, second, "two walks announced the same {name}");
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
