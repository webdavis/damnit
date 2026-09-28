mod command_line;
mod exchange;
mod pkce;
mod redirect;

use std::fmt;
use std::net::TcpListener;

use crate::{Endpoints, Secret};

pub use command_line::{client_id, client_secret};

const REDIRECT_HOST_KEEPING_THE_CODE_OFF_THE_NETWORK: &str = "127.0.0.1";

#[derive(Debug)]
pub struct Client {
    pub id: String,
    pub secret: Secret,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SignInError {
    Listener,
    Random,
    Redirect,
    State,
    Denied(&'static str),
    NoCode,
    Exchange {
        status: Option<u16>,
        code: Option<&'static str>,
    },
    NoRefreshToken,
}

impl fmt::Display for SignInError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SignInError::Listener => f.write_str(
                "no loopback port could be opened for the redirect, so nothing was asked for",
            ),
            SignInError::Random => {
                f.write_str("no random bytes could be read, so nothing was asked for")
            }
            SignInError::Redirect => {
                f.write_str("the browser redirect did not arrive, so no code was exchanged")
            }
            SignInError::State => {
                f.write_str("the redirect did not carry this run's state, so it was not answered")
            }
            SignInError::Denied(word) => {
                write!(f, "the consent was refused in the browser ({word})")
            }
            SignInError::NoCode => {
                f.write_str("the redirect carried no authorization code, so nothing was exchanged")
            }
            SignInError::Exchange { status: None, .. } => f.write_str(
                "the code exchange did not reach Google or its answer could not be read",
            ),
            SignInError::Exchange {
                status: Some(s),
                code: None,
            } => write!(f, "Google refused the code exchange (HTTP {s})"),
            SignInError::Exchange {
                status: Some(s),
                code: Some(c),
            } => write!(f, "Google refused the code exchange (HTTP {s}, {c})"),
            SignInError::NoRefreshToken => {
                f.write_str("Google answered the exchange with no refresh token")
            }
        }
    }
}

impl std::error::Error for SignInError {}

pub struct SignIn {
    agent: ureq::Agent,
    endpoints: Endpoints,
}

impl SignIn {
    pub fn new(endpoints: Endpoints) -> SignIn {
        SignIn {
            agent: crate::http::agent(),
            endpoints,
        }
    }

    pub fn mint(
        &self,
        client: &Client,
        announce: &mut dyn FnMut(&str),
    ) -> Result<Secret, SignInError> {
        let verifier = pkce::random_token()?;
        let state = pkce::random_token()?;
        let listener = TcpListener::bind((REDIRECT_HOST_KEEPING_THE_CODE_OFF_THE_NETWORK, 0))
            .map_err(|_| SignInError::Listener)?;
        let port = listener
            .local_addr()
            .map_err(|_| SignInError::Listener)?
            .port();
        let redirect_uri =
            format!("http://{REDIRECT_HOST_KEEPING_THE_CODE_OFF_THE_NETWORK}:{port}");
        announce(&redirect::authorization_url(
            &self.endpoints,
            &client.id,
            &redirect_uri,
            &pkce::s256_challenge(&verifier),
            &state,
        ));
        let code = redirect::await_the_one_browser_redirect(&listener, &state)?;
        exchange::exchange(
            &self.agent,
            &self.endpoints.token,
            client,
            &code,
            &verifier,
            &redirect_uri,
        )
    }
}
