mod loopback;
mod support;
mod walk;

use std::collections::HashMap;

use dam_remote_gcal::SignInError;
use loopback::Reply;
use walk::{
    CLIENT_SECRET, REFRESH, client, play_the_browser_returning_its_page, query_of,
    s256_computed_apart_from_the_code_under_test, sign_in_against, still_encoded_field,
};

#[test]
fn a_granted_consent_is_exchanged_for_the_refresh_token() {
    let _guard = support::guard("a_granted_consent_is_exchanged_for_the_refresh_token");
    let mut routes = HashMap::new();
    routes.insert("POST /token", vec![Reply::json(200, serde_json::json!({
        "access_token": "ya29.ACCESS", "expires_in": 3599, "refresh_token": REFRESH,
        "scope": "https://www.googleapis.com/auth/calendar.events.readonly", "token_type": "Bearer"
    }))]);
    let google = loopback::serve(routes);
    let mut browser = None;
    let mut announced = String::new();
    let token = sign_in_against(&google)
        .mint(&client(), &mut |url| {
            announced = url.to_string();
            browser = Some(play_the_browser_returning_its_page(url, |state| {
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
    let asked = loopback::decoded_fields(query_of(&announced));
    let sent = |name: &str| seen[0].decoded_form.get(name).cloned();
    assert_eq!(sent("grant_type").as_deref(), Some("authorization_code"));
    assert_eq!(sent("code").as_deref(), Some("4/0AbCODE"));
    assert_eq!(
        sent("client_id").as_deref(),
        Some("123.apps.googleusercontent.com")
    );
    assert_eq!(sent("client_secret").as_deref(), Some(CLIENT_SECRET));
    assert_eq!(sent("redirect_uri"), asked.get("redirect_uri").cloned());
    assert_eq!(
        still_encoded_field(&seen[0].body, "redirect_uri"),
        still_encoded_field(query_of(&announced), "redirect_uri"),
        "the exchange's redirect_uri must be the announced one, byte for byte"
    );
    let verifier = sent("code_verifier").unwrap_or_default();
    assert_eq!(
        Some(s256_computed_apart_from_the_code_under_test(&verifier)),
        asked.get("code_challenge").cloned(),
        "the verifier sent does not answer the challenge announced"
    );
    assert_eq!(
        seen[0].decoded_form.len(),
        6,
        "{:?}",
        seen[0].decoded_form.keys()
    );
}

#[test]
fn the_tests_own_s256_is_the_rfc_7636_appendix_b_example() {
    assert_eq!(
        s256_computed_apart_from_the_code_under_test("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
        "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
    );
}

#[test]
fn each_walk_announces_its_own_state_and_challenge() {
    let _guard = support::guard("each_walk_announces_its_own_state_and_challenge");
    let google = loopback::serve(HashMap::new());
    let sign_in = sign_in_against(&google);
    let mut announced = Vec::new();
    for _ in 0..2 {
        let mut browser = None;
        let _ = sign_in.mint(&client(), &mut |url| {
            announced.push(loopback::decoded_fields(query_of(url)));
            browser = Some(play_the_browser_returning_its_page(url, |state| {
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
    let mut browser = None;
    let err = sign_in_against(&google)
        .mint(&client(), &mut |url| {
            browser = Some(play_the_browser_returning_its_page(url, |state| {
                format!("error=access_denied&state={state}")
            }));
        })
        .unwrap_err();
    let page = browser.unwrap().join().unwrap();
    assert_eq!(err, SignInError::Denied("access_denied"));
    assert!(page.contains("refused"), "{page}");
    assert!(google.seen().is_empty());
}
