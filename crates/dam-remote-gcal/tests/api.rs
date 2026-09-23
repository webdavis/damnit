mod loopback;
mod support;

use std::collections::HashMap;

use dam_remote_gcal::access::access_token;
use dam_remote_gcal::{ApiError, Client, Credentials, Endpoints, http_agent};
use loopback::Reply;

const CLIENT_SECRET: &str = "GOCSPX-SUPERSECRETCLIENT";
const REFRESH: &str = "1//0gSUPERSECRETREFRESH";

fn credentials() -> Credentials {
    Credentials {
        client: Client {
            id: "123.apps".into(),
            secret: CLIENT_SECRET.into(),
        },
        refresh_token: REFRESH.into(),
    }
}

fn token_route(reply: Reply) -> loopback::Loopback {
    let mut routes = HashMap::new();
    routes.insert("POST /token", vec![reply]);
    loopback::serve(routes)
}

#[test]
fn the_refresh_grant_sends_the_three_credentials_in_a_form_and_reads_the_access_token() {
    let _guard = support::guard(
        "the_refresh_grant_sends_the_three_credentials_in_a_form_and_reads_the_access_token",
    );
    let google = token_route(Reply::json(
        200,
        serde_json::json!({"access_token": "ya29.ACCESS", "expires_in": 3599, "token_type": "Bearer"}),
    ));
    let token = access_token(
        &http_agent(),
        &Endpoints::loopback(&google.base).unwrap(),
        &credentials(),
    )
    .unwrap();
    assert_eq!(token.expose(), "ya29.ACCESS");
    let body = &google.seen()[0].body;
    for field in [
        "grant_type=refresh_token",
        "client_id=123.apps",
        "client_secret=GOCSPX-SUPERSECRETCLIENT",
        "refresh_token=1%2F%2F0gSUPERSECRETREFRESH",
    ] {
        assert!(body.contains(field), "{field}");
    }
}

#[test]
fn a_revoked_refresh_token_says_to_sign_in_again_and_quotes_nothing() {
    let _guard = support::guard("a_revoked_refresh_token_says_to_sign_in_again_and_quotes_nothing");
    let google = token_route(Reply::json(
        400,
        serde_json::json!({
            "error": "invalid_grant", "error_description": format!("Token has been expired or revoked. {REFRESH} {CLIENT_SECRET}")
        }),
    ));
    let err = access_token(
        &http_agent(),
        &Endpoints::loopback(&google.base).unwrap(),
        &credentials(),
    )
    .unwrap_err();
    let said = err.to_string();
    assert!(
        said.contains("invalid_grant") && said.contains("dam-gcal-sign-in"),
        "{said}"
    );
    assert!(
        said.contains("\"Testing\" expires it after 7 days"),
        "{said}"
    );
    for secret in [REFRESH, CLIENT_SECRET, "expired or revoked"] {
        assert!(!said.contains(secret), "{said}");
    }
}

#[test]
fn an_answer_without_an_access_token_is_refused_without_quoting_it() {
    let _guard = support::guard("an_answer_without_an_access_token_is_refused_without_quoting_it");
    let google = token_route(Reply::json(
        200,
        serde_json::json!({"token_type": "Bearer", "echo": REFRESH}),
    ));
    let err = access_token(
        &http_agent(),
        &Endpoints::loopback(&google.base).unwrap(),
        &credentials(),
    )
    .unwrap_err();
    assert!(
        matches!(
            err,
            ApiError::TokenRefused {
                status: Some(200),
                code: None
            }
        ),
        "{err:?}"
    );
    assert!(!err.to_string().contains(REFRESH));
}

/// An empty token is no token: sent as a bearer it would fail every call
/// after it with a message about the calendar rather than the grant.
#[test]
fn an_empty_access_token_is_refused() {
    let _guard = support::guard("an_empty_access_token_is_refused");
    let google = token_route(Reply::json(
        200,
        serde_json::json!({"access_token": "", "token_type": "Bearer"}),
    ));
    let err = access_token(
        &http_agent(),
        &Endpoints::loopback(&google.base).unwrap(),
        &credentials(),
    )
    .unwrap_err();
    assert!(
        matches!(
            err,
            ApiError::TokenRefused {
                status: Some(200),
                code: None
            }
        ),
        "{err:?}"
    );
}

#[test]
fn a_token_endpoint_nobody_answers_is_an_exchange_that_did_not_reach_google() {
    let _guard =
        support::guard("a_token_endpoint_nobody_answers_is_an_exchange_that_did_not_reach_google");
    let closed = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", closed.local_addr().unwrap());
    drop(closed);
    let err = access_token(
        &http_agent(),
        &Endpoints::loopback(&base).unwrap(),
        &credentials(),
    )
    .unwrap_err();
    assert!(
        matches!(
            err,
            ApiError::TokenRefused {
                status: None,
                code: None
            }
        ),
        "{err:?}"
    );
    assert_eq!(
        err.to_string(),
        "the token exchange did not reach Google or its answer could not be read"
    );
}

#[test]
fn an_error_never_debug_prints_a_credential() {
    let _guard = support::guard("an_error_never_debug_prints_a_credential");
    let google = token_route(Reply::json(
        401,
        serde_json::json!({"error": "invalid_client", "e": CLIENT_SECRET}),
    ));
    let err = access_token(
        &http_agent(),
        &Endpoints::loopback(&google.base).unwrap(),
        &credentials(),
    )
    .unwrap_err();
    assert!(!format!("{err:?}").contains(CLIENT_SECRET));
}
