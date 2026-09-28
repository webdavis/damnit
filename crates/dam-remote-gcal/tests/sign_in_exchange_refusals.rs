mod loopback;
mod support;
mod walk;

use std::collections::HashMap;

use dam_remote_gcal::SignInError;
use loopback::Reply;
use walk::{CLIENT_SECRET, REFRESH, mint_with_the_browser_granting_code_c};

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
    let err = mint_with_the_browser_granting_code_c(&google).unwrap_err();
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
        let err = mint_with_the_browser_granting_code_c(&google).unwrap_err();
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
    let err = mint_with_the_browser_granting_code_c(&google).unwrap_err();
    assert_eq!(
        err,
        SignInError::Exchange {
            status: None,
            code: None
        }
    );
    assert!(err.to_string().contains("could not be read"), "{err}");
}

#[test]
fn a_redirect_from_the_token_endpoint_is_its_answer_so_no_credential_follows_it() {
    let _guard = support::guard(
        "a_redirect_from_the_token_endpoint_is_its_answer_so_no_credential_follows_it",
    );
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
    let err = mint_with_the_browser_granting_code_c(&google).unwrap_err();
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
