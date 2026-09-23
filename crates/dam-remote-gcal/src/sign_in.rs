//! The one-time consent walk that mints the refresh token a gcal remote reads.
//!
//! Google's installed-application flow: a loopback redirect on an ephemeral
//! port, PKCE with S256, and `access_type=offline` with `prompt=consent`, which
//! is what makes the exchange answer with a refresh token. The credentials
//! travel in the exchange's request body and nowhere else, and every refusal
//! is a fixed sentence naming the step.

mod command_line;
mod exchange;
mod pkce;
mod redirect;

use std::fmt;
use std::net::TcpListener;

use crate::{Endpoints, Secret};

pub use command_line::{client_id, client_secret};

/// The address the redirect comes back to. Loopback only: the code is a
/// credential, and a listener on any other interface offers it to the network.
const REDIRECT_HOST: &str = "127.0.0.1";

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

    /// One consent: the URL handed to `announce` for the operator to open, the
    /// single redirect read off a loopback port, and the code exchanged. The
    /// verifier and the state are minted here, so no caller can reuse one.
    pub fn mint(
        &self,
        client: &Client,
        announce: &mut dyn FnMut(&str),
    ) -> Result<Secret, SignInError> {
        let verifier = pkce::random_token()?;
        let state = pkce::random_token()?;
        let listener = TcpListener::bind((REDIRECT_HOST, 0)).map_err(|_| SignInError::Listener)?;
        let port = listener
            .local_addr()
            .map_err(|_| SignInError::Listener)?
            .port();
        let redirect_uri = format!("http://{REDIRECT_HOST}:{port}");
        announce(&redirect::authorization_url(
            &self.endpoints,
            &client.id,
            &redirect_uri,
            &pkce::challenge(&verifier),
            &state,
        ));
        let code = redirect::await_code(&listener, &state)?;
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
